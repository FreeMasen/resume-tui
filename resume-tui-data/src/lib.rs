use std::collections::LinkedList;

use serde::{Deserialize, Serialize};

#[cfg(feature = "codegen")]
pub mod codegen;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database<T> {
    pub name: T,
    pub tag_line: T,
    pub jobs: LinkedList<Workplace<T>>,
    pub open_source: LinkedList<()>,
    pub education: LinkedList<()>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workplace<T> {
    pub name: T,
    pub title: T,
    pub start: T,
    pub end: Option<T>,
    pub details: LinkedList<Detail<T>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detail<T> {
    pub short: T,
    pub long: T,
    pub detail: T,
}
