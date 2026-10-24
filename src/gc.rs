use crate::heap::Heap;
use crate::heap::Object;
use crate::value::Value;
use std::collections::HashSet;
use std::time::Instant;

pub struct GarbageCollector {
    marked: HashSet<u32>,
    debug: bool,
}

impl GarbageCollector {
    pub fn new(debug: bool) -> Self {
        Self {
            marked: HashSet::new(),
            debug,
        }
    }

    pub fn collect(&mut self, heap: &mut Heap, roots: &[&Value]) {
        let start = Instant::now();

        self.marked.clear();

        for root in roots {
            self.mark_value(heap, root);
        }

        if self.debug {
            println!("(GC) DEBUG: Marked {} value(s)", self.marked.len())
        }

        let objects: Vec<u32> = heap.ids().collect();

        let mut count = 0;
        for id in objects {
            if !self.marked.contains(&id) {
                heap.remove(id);
                count += 1;
            }
        }

        if self.debug {
            println!("(GC) DEBUG: Swept {} object(s)", count);
            println!("(GC) DEBUG: Took {:?}", start.elapsed());
        }

        heap.set_allocations_since_gc(0);
    }

    fn mark_value(&mut self, heap: &Heap, value: &Value) {
        let id = match value {
            Value::ObjectRef(id) => *id,
            _ => return,
        };

        self.mark_object(heap, id);
    }

    fn mark_object(&mut self, heap: &Heap, id: u32) {
        if !self.marked.insert(id) {
            return;
        }

        let object = match heap.get(id) {
            Ok(obj) => obj,
            Err(_) => return,
        };

        match object {
            Object::Array(vec) => {
                for val in vec {
                    self.mark_value(heap, &val);
                }
            }

            Object::Map(map) => {
                for (key, val) in map {
                    self.mark_value(heap, &key);
                    self.mark_value(heap, &val);
                }
            }

            Object::Closure(_, _, captures) => {
                for capture in captures {
                    self.mark_value(heap, capture);
                }
            }

            _ => {}
        }
    }
}
