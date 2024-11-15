use bitcoind;

pub fn get_bitcoin_instance() -> Result<bitcoind::BitcoinD, Box<dyn std::error::Error>> {
    bitcoind::exe_path().map_or_else(
        |_| {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "BitcoinD executable not found",
            )) as Box<dyn std::error::Error>)
        },
        |exe_path| {
            let bitcoind = bitcoind::BitcoinD::new(exe_path).unwrap();
            Ok(bitcoind)
        },
    )
}
