fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", oxevu_api::contract::openapi().to_pretty_json()?);
    Ok(())
}
