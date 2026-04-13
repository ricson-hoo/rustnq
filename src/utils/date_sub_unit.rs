use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize,Clone)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum DateSubUnit {
    YEAR,
    MONTH,
    DAY,
}

impl From<DateSubUnit> for String {
    fn from(item: DateSubUnit) -> Self {
        match item {
            DateSubUnit::YEAR => "YEAR".to_string(),
            DateSubUnit::MONTH => "MONTH".to_string(),
            DateSubUnit::DAY => "DAY".to_string(),
        }
    }
}

impl From<&str> for DateSubUnit {
    fn from(s: &str) -> Self {
        match s {
            "YEAR" => DateSubUnit::YEAR,
            "MONTH" => DateSubUnit::MONTH,
            "DAY" => DateSubUnit::DAY,
            &_ => todo!(),
        }
    }
}
impl fmt::Display for DateSubUnit {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DateSubUnit::YEAR => write!(f,"YEAR"),
            DateSubUnit::MONTH => write!(f,"MONTH"),
            DateSubUnit::DAY => write!(f,"DAY"),
        }
    }
}