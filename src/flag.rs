// Copyright Thomas Polasek
pub struct Flag {
    pub flag_n: u8,
    pub flag_v: u8,
    pub flag_b: u8,
    pub flag_d: u8,
    pub flag_i: u8,
    pub flag_z: u8,
    pub flag_c: u8
}
impl Flag {
    pub fn get_sr(&self) -> u8{
        return (self.flag_c << 0) | (self.flag_z << 1) | (self.flag_i << 2) | (self.flag_d << 3) | (self.flag_b << 4) | (1 << 5) | (self.flag_v << 6) | (self.flag_n << 7);
    }

    pub fn set_sr(&mut self, value : u8){
        self.flag_c = value & (1 << 0);
        self.flag_z = value & (1 << 1);
        self.flag_i = value & (1 << 2);
        self.flag_d = value & (1 << 3);
        self.flag_b = value & (1 << 4);
        // 5 is empty
        self.flag_v = value & (1 << 6);
        self.flag_n = value & (1 << 7);

    }

    pub fn bool_to_u8(&self, set : bool) -> u8 {
        return match set {
            true => 1,
            false => 0
        };
    }

    pub fn set_flag_n(&mut self, set : bool) { self.flag_n = self.bool_to_u8(set); }
    pub fn set_flag_v(&mut self, set : bool) { self.flag_v = self.bool_to_u8(set); }
    pub fn set_flag_b(&mut self, set : bool) { self.flag_b = self.bool_to_u8(set); }
    pub fn set_flag_d(&mut self, set : bool) { self.flag_d = self.bool_to_u8(set); }
    pub fn set_flag_i(&mut self, set : bool) { self.flag_i = self.bool_to_u8(set); }
    pub fn set_flag_z(&mut self, set : bool) { self.flag_z = self.bool_to_u8(set); }
    pub fn set_flag_c(&mut self, set : bool) { self.flag_c = self.bool_to_u8(set); }

    pub fn get_flag_n(&self) -> bool { assert!(self.flag_n <= 1); return self.flag_n == 1; }
    pub fn get_flag_v(&self) -> bool { assert!(self.flag_v <= 1); return self.flag_v == 1; }
    pub fn get_flag_b(&self) -> bool { assert!(self.flag_b <= 1); return self.flag_b == 1; }
    pub fn get_flag_d(&self) -> bool { assert!(self.flag_d <= 1); return self.flag_d == 1; }
    pub fn get_flag_i(&self) -> bool { assert!(self.flag_i <= 1); return self.flag_i == 1; }
    pub fn get_flag_z(&self) -> bool { assert!(self.flag_z <= 1); return self.flag_z == 1; }
    pub fn get_flag_c(&self) -> bool { assert!(self.flag_n <= 1); return self.flag_c == 1; }

    // NV-BDIZC
    pub fn get_formatted_str(&self) -> String{
        let mut output = String::with_capacity(10);
        output.push_str("[ ");
        if self.get_flag_n() { output.push_str("N "); }
        if self.get_flag_v() { output.push_str("V "); }
        if self.get_flag_b() { output.push_str("B "); }
        if self.get_flag_d() { output.push_str("D "); }
        if self.get_flag_i() { output.push_str("I "); }
        if self.get_flag_z() { output.push_str("Z "); }
        if self.get_flag_c() { output.push_str("C "); }
        output.push_str("]");
        return output;
    }
}