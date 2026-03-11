use crate::{
    pyrs_codeobject::PyCodeObj,
    pyrs_interpreter::PyRsVersion,
    pyrs_utils::{FromBytes, PyUtils},
};

#[derive(Debug, Clone, PartialEq)]
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
        //println!("{:?}", &vec);

        // n bytes
        for c in self.name.as_bytes() {
            vec.push(*c);
        }
        vec.push(0);
        //println!("{:?}", &vec);

        // 2 bytes
        for c in self.time.to_be_bytes() {
            vec.push(c);
        }
        //println!("{:?}", &vec);

        // 3 bytes
        vec.push(self.version.major);
        vec.push(self.version.minor);
        vec.push(self.version.patch);
        //println!("{:?}", &vec);

        // n bytes
        for c in self.internal_filename.as_bytes() {
            vec.push(*c);
        }
        vec.push(0);
        //println!("{:?}", &vec);

        vec
    }

    pub fn deserialize(bytes: &Vec<u8>) -> Self {
        let s = unsafe { String::from_utf8_unchecked(bytes.clone()) };
        let starter = &s[0..4];
        assert_eq!(starter, "pyrs");

        let mut i = 0;
        for c in s.chars() {
            if c == '\0' {
                break;
            }
            i += 1;
        }

        let name = &s[4..i];
        i += 1; // skip '\0'

        let time_bytes = s[i..i + 8].as_bytes().to_vec();
        dbg!(&time_bytes);
        let time_num = u64::from_bytes_be(time_bytes.as_slice()).unwrap();

        let vers = s[i + 8..i + 12].as_bytes().to_vec();

        let mut filename = String::new();
        i += 11;
        for x in s[i..s.len()].chars() {
            //i += 1;
            if x == '\0' {
                break;
            }
            filename.push(x);
        }

        PyHeader {
            name: name.to_string(),
            time: time_num,
            version: PyRsVersion {
                major: vers[0],
                minor: vers[1],
                patch: vers[2],
            },
            internal_filename: filename,
        }
    }
}

pub struct PySerializer {}

impl PySerializer {
    pub fn seralize_codeobj(code_obj: &PyCodeObj) -> Vec<u8> {
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

        /*
        let len = bytecode[0].get_type_str().len();
        println!("len: \n{}", len);

        let mut bytecode_map = [[b'_'; PyBytecode::TYPE_STR_LEN]; 255];
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
        let inst_list_len_bytes = (inst_list.len() * 2).to_be_bytes().to_vec();
        assert_eq!(inst_list_len_bytes.len(), 8);
        */

        // final
        let mut final_bytes: Vec<u8> = vec![];

        final_bytes.append(
            &mut PyHeader {
                name: name.to_string(),
                time: PyUtils::curr_time(),
                version: PyRsVersion::get(),
                internal_filename: PyUtils::curr_dir(),
            }
            .seralize(),
        );

        let csts = "__consts__";
        final_bytes.append(&mut csts.to_string().into_bytes());
        final_bytes.push(0); // null term
        final_bytes.append(&mut (consts.len() as u64).to_be_bytes().to_vec()); // 8 bytes of num_consts
        for c in consts {
            let obj_str = c.get_ref().__str__();
            final_bytes.append(&mut (obj_str.len() as u64).to_be_bytes().to_vec()); // 8 bytes of obj len = n
            final_bytes.append(&mut obj_str.into_bytes()); // n bytes of obj string
        }

        let vrnms = "__varnames__";
        final_bytes.append(&mut vrnms.to_string().into_bytes()); // string
        final_bytes.push(0); // null term
        final_bytes.append(&mut (varnames.len() as u64).to_be_bytes().to_vec()); // 8 bytes of num_varnames
        for v in varnames {
            final_bytes.append(&mut (v.len() as u64).to_be_bytes().to_vec()); // n bytes of obj string
            final_bytes.append(&mut v.clone().into_bytes()); // n bytes of obj string
        }

        let nms = "__names__";
        final_bytes.append(&mut nms.to_string().into_bytes()); // string
        final_bytes.push(0); // null term
        final_bytes.append(&mut (names.len() as u64).to_be_bytes().to_vec()); // 8 bytes of num_names
        for n in names {
            final_bytes.append(&mut (n.len() as u64).to_be_bytes().to_vec()); // n bytes of obj string
            final_bytes.append(&mut n.clone().into_bytes()); // n bytes of obj string
        }

        let bcde = "__bytecode__";
        final_bytes.append(&mut bcde.to_string().into_bytes()); // string
        final_bytes.push(0); // null term
        final_bytes.append(&mut (bytecode.len() as u64).to_be_bytes().to_vec()); // 8 bytes of num_names
        for inst in bytecode {
            // <"__bytecode__">, <inst len as u64 (8 bytes)>, <instructions (2 bytes per inst) * n>
            final_bytes.append(&mut inst.to_bytes().to_vec());
        }

        final_bytes
    }

    pub fn deserialize_codeobj(bytes: Vec<u8>) -> PyCodeObj {
        // final
        let _header = PyHeader::deserialize(&bytes);

        /*
        CodeObj {
            name: ,
            bytecode: ,
            consts: ,
            names: ,
            varnames: ,
            num_consts: ,
            num_varnames: ,
            num_names:
        }
        */
        PyCodeObj::new("__empty__", vec![])
    }
}
