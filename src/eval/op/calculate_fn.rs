mod trig;
mod logs;
use crate::eval::Number;
use std::convert::TryFrom;

pub trait CalculateFn {
    type Output;
    fn calculate_fn(self, fn_name: &String) -> Self::Output;
}

impl CalculateFn for Number {
    type Output = Result<Self, String>;
    fn calculate_fn(self, fn_name: &String) -> Self::Output {
       let f = FunctionKind::try_from(fn_name).map(|fk| fk.as_function())?;
       f(self)
    }
}

#[derive(Clone, Copy)]
enum FunctionKind {
    Sin,
    Cos,
    Tan,
    Log10,
    Ln
}

impl TryFrom<&String> for FunctionKind {
    type Error = String;
    fn try_from(word: &String) -> Result<Self, Self::Error> {
        Ok(match word.as_str() {
            "sin" => Self::Sin,
            "cos" => Self::Cos,
            "tan" => Self::Tan,
            "log" => Self::Log10,
            "ln" => Self::Ln,
            unknown => {return Err(format!("unknown function: {}", unknown))}
        })
    }
    
}

// type Function = impl FnOnce(Data) -> Result<Data, String>;

impl FunctionKind {
    fn as_function(&self) -> impl FnOnce(Number) -> Result<Number, String>  {
        match self {
            Self::Sin => |x| self::trig::sin(x),
            Self::Cos => |x| self::trig::cos(x),
            Self::Tan => |x| self::trig::tan(x),
            Self::Ln => |x| self::logs::natural_log(x),
            Self::Log10 => |x| self::logs::log_10(x),
        }
    }
}
