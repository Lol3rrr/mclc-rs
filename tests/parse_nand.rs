#[test]
fn parse_graph() {
    let content = "
entity NandGate {
  in_ports {
    src1 : bit;
    src2 : bit;
  }

  out_ports {
    result : bit;
  }

  behaviour {
    (tmp) = and (src1, src2);
    (result) <= not(tmp);
  }
} 
        ";

    let result = mclc::frontend::parse(content, None);

    dbg!(&result);
    todo!()
}
