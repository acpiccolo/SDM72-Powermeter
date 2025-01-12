use crate::protocol as proto;

pub const DATA_BITS: &tokio_serial::DataBits = &tokio_serial::DataBits::Eight;

pub fn serial_port_builder(
    device: &String,
    baud_rate: &proto::BaudRate,
    parity_and_stop_bits: &proto::ParityAndStopBit,
) -> tokio_serial::SerialPortBuilder {
    let (parity, stop_bits) = match parity_and_stop_bits {
        proto::ParityAndStopBit::NoParityOneStopBit => {
            (tokio_serial::Parity::None, tokio_serial::StopBits::One)
        }
        proto::ParityAndStopBit::EvenParityOneStopBit => {
            (tokio_serial::Parity::Even, tokio_serial::StopBits::One)
        }
        proto::ParityAndStopBit::OddParityOneStopBit => {
            (tokio_serial::Parity::Odd, tokio_serial::StopBits::One)
        }
        proto::ParityAndStopBit::NoParityTwoStopBits => {
            (tokio_serial::Parity::None, tokio_serial::StopBits::Two)
        }
    };
    tokio_serial::new(device, u16::from(baud_rate) as u32)
        .parity(parity)
        .stop_bits(stop_bits)
        .data_bits(*DATA_BITS)
        // .timeout(timeout) // Do not work, set it to the context
        .flow_control(tokio_serial::FlowControl::None)
}
