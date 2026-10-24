use crate::error::CandleResult;
use crate::value::Value;
use crate::vm::Vm;

impl Vm {
    pub fn exec_add(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, (a_val + b_val)?))
    }

    pub fn exec_sub(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, (a_val - b_val)?))
    }

    pub fn exec_mul(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, (a_val * b_val)?))
    }

    pub fn exec_div(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, (a_val / b_val)?))
    }

    pub fn exec_mod(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, (a_val % b_val)?))
    }

    pub fn exec_inc(&mut self) -> CandleResult<()> {
        let reg = self.fetch_u8()? as usize;
        let val = self.get_register(reg);

        Ok(self.set_register(reg, (val + Value::Number(1.0))?))
    }

    pub fn exec_dec(&mut self) -> CandleResult<()> {
        let reg = self.fetch_u8()? as usize;
        let val = self.get_register(reg);
        Ok(self.set_register(reg, (val - Value::Number(1.0))?))
    }
}
