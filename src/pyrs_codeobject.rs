
#[allow(unused)]
mod numbers {

use crate::pyrs_bytecode::{PyBytecode};
use rug::{Integer};
use std::{
    collections::{HashSet},
    hash::Hash,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CodeObject
{
    co_nlocals: usize,
    co_argcount: usize,
    co_varnames: Vec<String>,
    co_names: Vec<String>,
    co_freevars: Vec<String>,
    co_cellvars: Vec<String>,
    co_posonlyargcount: usize,
    co_kwonlyargcount: usize,
    co_firstlineno: usize,
    co_lnotab: usize,
    co_stacksize: usize,
    co_code: Vec<PyBytecode>,
    co_consts: Vec<String>,
    co_flags: usize,
}

pub struct PythonObject<T: PyObject>
{
    id_: usize,
    type_: usize,
    value_: T,
}

impl<T: PyObject> PythonObject<T>
{
    pub fn is(lhs: &Self, rhs: &Self) -> bool {
        lhs.id_ == rhs.id_
    }
    pub fn id(&self) -> usize {
        self.type_
    }
}

pub trait PyObject
{
    fn __type__(&self) -> &str;
    fn bool(&self) -> PyBool { PyBool(false) }
}

pub trait PyNumber: Sized
{
    fn __zero__() -> Self;
    fn __add__(&self, other: &Self) -> Self;
}

pub trait PyReal: Sized
{
    fn float(&self) -> PyFloat;
}

pub trait PyIntegral: Sized
{
    fn int(&self) -> PyInt;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PyNone { }

impl PyObject for PyNone {
    fn __type__(&self) -> &str {
        "NoneType"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PyNotImplemented {}

impl PyObject for PyNotImplemented {
    fn __type__(&self) -> &str {
        "NotImplementedType"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PyEllipsis {}

impl PyObject for PyEllipsis {
    fn __type__(&self) -> &str {
        "ellipsis"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PyFloat(f64);

impl PyObject for PyFloat {
    fn __type__(&self) -> &str {
        "float"
    }
    fn bool(&self) -> PyBool {
        PyBool(self.0 != 0.0 && !self.0.is_nan())
    }
}

impl PyNumber for PyFloat {
    fn __zero__() -> Self {
        PyFloat(0.0)
    }
    fn __add__(&self, other: &Self) -> Self {
        PyFloat(self.0 + other.0)
    }
}

impl PyReal for PyFloat {
    fn float(&self) -> PyFloat {
        *self
    }
}

impl PyIntegral for PyFloat {
    fn int(&self) -> PyInt {
        PyInt(Integer::from(self.0 as i64))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PyInt(Integer);

impl PyObject for PyInt {
    fn __type__(&self) -> &str {
        "int"
    }
    fn bool(&self) -> PyBool {
        PyBool(self.0 != 0)
    }
}

impl PyNumber for PyInt {
    fn __zero__() -> Self {
        PyInt(Integer::ZERO)
    }
    fn __add__(&self, other: &Self) -> Self {
        PyInt(self.0.clone() + other.0.clone())
    }
}

impl PyReal for PyInt {
    fn float(&self) -> PyFloat {
        PyFloat(self.0.to_f64())
    }
}

impl PyIntegral for PyInt {
    fn int(&self) -> PyInt {
        self.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PyBool(bool);

impl PyObject for PyBool {
    fn __type__(&self) -> &str {
        "bool"
    }
    fn bool(&self) -> PyBool {
        PyBool(self.0)
    }
}

impl PyNumber for PyBool {
    fn __zero__() -> Self {
        PyBool(false)
    }
    fn __add__(&self, other: &Self) -> Self {
        PyBool(self.0 || other.0)
    }
}

impl PyReal for PyBool {
    fn float(&self) -> PyFloat {
        PyFloat(if self.0 { 1.0 } else { 0.0 })
    }
}

impl PyIntegral for PyBool {
    fn int(&self) -> PyInt {
        PyInt(Integer::from(if self.0 { 1 } else { 0 }))
    }
}
 
}
