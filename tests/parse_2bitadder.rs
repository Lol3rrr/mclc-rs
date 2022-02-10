#[test]
fn parse_2bitadder() {
    let content = include_str!("./files/2BitAdder.mcl");

    let result = mclc::frontend::parse(content, None);
    dbg!(&result);

    todo!()
}
