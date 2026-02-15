

use crate::{
    pyrs_bytecode::PyBytecode, pyrs_codeobject::CodeObj, pyrs_interpreter::PyRsVersion
};

pub struct PyHeader {
    pub name: String,
    pub time: u64,
    pub version: PyRsVersion,
    pub internal_filename: String,
}

impl PyHeader {
    
    // Header
    // <"pyrs"> <name (null_terminated)> <compile_time> <pyrs_version(u8 + u8 + u8)> <internal_filename>
    // 
    pub fn seralize(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = vec![];

        // 4 bytes
        let starter = String::from("pyrs");
        for c in starter.as_bytes() {
            vec.push(*c);
        }
        vec.push(0);

        // n bytes
        for c in self.name.as_bytes() {
            vec.push(*c);
        }

        // 2 bytes
        for c in self.time.to_le_bytes() {
            vec.push(c);
        }
        
        // 3 bytes
        vec.push(self.version.major);
        vec.push(self.version.minor);
        vec.push(self.version.patch);

        // n bytes
        for c in self.internal_filename.as_bytes() {
            vec.push(*c);
        }
        vec.push(0);

        vec
    }

    pub fn deserialize(bytes: Vec<u8>) -> Self {
        let s = String::from_utf8(bytes).unwrap();
        println!("{}", s);
        panic!();
    }

}

pub struct PySerializer {}

impl PySerializer {

    pub fn seralize_codeobj(code_obj: &CodeObj) -> String {
        // byte for instruction
        // byte for number

        // some instructions have args after

        let name = &code_obj.name;
        let consts = &code_obj.consts;
        let varnames = &code_obj.varnames;
        let names = &code_obj.names;
        let bytecode = &code_obj.bytecode;

        // metadata: 
        //      num bytecode instructions
        
        // constant_element_len = 16 chars
        // element = <name/Empty>_..._ (16)
        let len = bytecode[0].get_type_str().len();
        println!("len: \n{}", len);

        let mut bytecode_map = [[b'_';  PyBytecode::TYPE_STR_LEN]; 255];
        for i in 0u8..255u8 {
            let index = i as usize;
            bytecode_map[index] = PyBytecode::from_bytes(&[i, 0]).get_type_str_slice().clone();
        }

        // bytecode
        // <1: instruction> <2: data>
        let mut inst_list: Vec<[u8; 2]> = vec![];
        inst_list.reserve(bytecode.len());
        for inst in bytecode {
            inst_list.push(inst.to_bytes());
        }
        let inst_list_len_bytes = (inst_list.len() * 2).to_le_bytes().to_vec();
        assert_eq!(inst_list_len_bytes.len(), 8);

        // final
        let mut final_bytes: Vec<u8> = vec![];
        final_bytes.append(&mut name.clone().into_bytes());
        final_bytes.push(b'\0');

        let mut final_str = String::new();
        final_str.push_str(name);
        final_str.push('\0');

        final_str.push_str("__consts__");
        for c in consts {
            final_str.push_str(c.repr());
            final_str.push('\0');
        }

        final_str.push_str("__varnames__");
        for v in varnames {
            final_str.push_str(v);
            final_str.push('\0');
        }

        // <"__names__">, <
        final_str.push_str("__names__");
        for v in names {
            final_str.push_str(v);
            final_str.push('\0');
        }

        // <"__bytecode__">, <inst len as u64 (8 bytes)>, <instructions (2 bytes per inst) * n>  
        final_str.push_str("__bytecode__");
        for b in inst_list_len_bytes {
            final_str.push(b as char);
        }
        for [a, b] in inst_list {
            final_str.push(a as char);
            final_str.push(b as char);
        }

        final_str
    }
}