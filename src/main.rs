use clap::Parser;

#[derive(Debug, Parser)]
struct Arguments {
    #[clap(name = "file")]
    file: String,
    #[clap(name = "target")]
    target: Option<String>,
}

fn main() {
    let args = Arguments::parse();
    dbg!(&args);

    let content = std::fs::read_to_string(args.file).unwrap();

    let mut graph = mclc::frontend::parse(content, args.target).unwrap();
    graph.optimize();

    println!("Generated Graph");

    let layout = mclc::backend::generate_layout(graph);

    println!("Generated Layout");

    layout.generate_svg("./placement.svg");
    let placement = layout.placement();

    let cmds = placement.place_commands();
    std::fs::write(
        "./commands.txt",
        cmds.into_iter().fold("".to_string(), |mut acc, c| {
            acc.push_str(&c);
            acc.push('\n');
            acc
        }),
    )
    .unwrap();

    println!("Done");
}
