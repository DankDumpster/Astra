use serde::{Deserialize, Serialize};

use crate::api::launch::Launch;
use crate::api::event::Event;

pub trait Ctx: std::fmt::Display {}

impl Ctx for str {}
impl Ctx for String {}
impl Ctx for i32 {}

pub trait ResObject {}

impl ResObject for Launch {}
impl ResObject for Event {}