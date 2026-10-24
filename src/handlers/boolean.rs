use crate::error::CandleResult;
use crate::vm::Vm;

impl Vm {
    pub fn exec_and(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, (a_val & b_val)?))
    }

    pub fn exec_or(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, (a_val | b_val)?))
    }

    pub fn exec_xor(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, (a_val ^ b_val)?))
    }

    pub fn exec_not(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let source_reg = self.fetch_u8()? as usize;

        let val = self.get_register(source_reg);

        Ok(self.set_register(target_reg, (!val)?))
    }
}
