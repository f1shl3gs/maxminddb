use std::net::IpAddr;

use maxminddb::City;

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let reader = maxminddb::Reader::open_file(
        args.next()
            .ok_or("First argument must be the path to the IP database")?,
    )
    .unwrap();
    let ip: IpAddr = args
        .next()
        .ok_or("Second argument must be the IP address, like 128.101.101.101")?
        .parse()
        .unwrap();
    let city: City = reader.lookup(ip).unwrap();
    println!("{city:#?}");
    Ok(())
}
