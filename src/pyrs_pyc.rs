
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::pyrs_codeobject::PyCodeObj;

pub struct PyPyc {}

#[allow(unused)]
impl PyPyc {

    fn serialize(code_obj: &PyCodeObj) {

        let mut hasher = DefaultHasher::new();
        code_obj.hash(&mut hasher);
        let hash_num = hasher.finish();
        
        //let header = vec![]

    }

}