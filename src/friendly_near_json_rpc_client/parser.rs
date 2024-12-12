use std::error::Error;

pub trait ParseResult: Sized {
    fn parse(result_str: String) -> Result<Self, Box<dyn Error>>;
}

impl ParseResult for Vec<u8> {
    fn parse(result_str: String) -> Result<Self, Box<dyn Error>> {
        let result_bytes: Vec<u8> = result_str
            .trim_matches(|c| c == '[' || c == ']') // Eliminar corchetes
            .split(',') // Dividir por comas
            .map(|s| s.trim().parse::<u8>().unwrap()) // Convertir cada parte a u8
            .collect();
        Ok(result_bytes)
    }
}

impl ParseResult for u128 {
    fn parse(result_str: String) -> Result<Self, Box<dyn Error>> {
        let value = result_str.trim_matches('"').parse::<u128>()?;
        Ok(value)
    }
}

impl ParseResult for String {
    fn parse(result_str: String) -> Result<Self, Box<dyn Error>> {
        Ok(result_str)
    }
}
