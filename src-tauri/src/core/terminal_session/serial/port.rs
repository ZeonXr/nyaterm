pub fn list_serial_ports() -> AppResult<Vec<String>> {
    let mut port_names = serialport::available_ports()
        .map_err(|e| AppError::Config(format!("Failed to list serial ports: {e}")))?
        .into_iter()
        .map(|port| port.port_name)
        .collect::<Vec<_>>();
    port_names.sort_unstable();
    Ok(port_names)
}

fn parse_data_bits(v: u8) -> DataBits {
    match v {
        5 => DataBits::Five,
        6 => DataBits::Six,
        7 => DataBits::Seven,
        _ => DataBits::Eight,
    }
}

fn parse_parity(v: &str) -> Parity {
    match v {
        "odd" => Parity::Odd,
        "even" => Parity::Even,
        _ => Parity::None,
    }
}

fn parse_stop_bits(v: &str) -> StopBits {
    match v {
        "2" => StopBits::Two,
        _ => StopBits::One,
    }
}
