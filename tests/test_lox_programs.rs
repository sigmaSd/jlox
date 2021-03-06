use jlox::{Lox, Result};

macro_rules! assert_test_eq {
    ($name: literal => $expected: literal) => {
        let output = run_test_with_output($name)?;
        assert_eq!(output, $expected, $name);
    };
}

macro_rules! test_lox_programs  {
    ($($name: ident)+) => {
        $(
        #[test]
        fn $name() -> Result<()> {
            let mut lox = Lox::default();
            lox.run_file(format!("lox_files/{}.lox", stringify!($name)))
        }
        )+
    }
}

test_lox_programs!(hello env fib fun hidden_var fact closure_scope class instance run_class_method class_cake init inherit ssuper);

#[test]
fn test_lox_programs() -> Result<()> {
    assert_test_eq!("ssuper" => "Fry until golden brown.\nPipe full of custard and coat with chocolate.\n");
    assert_test_eq!("inherit" => "Fry until golden brown.\n");
    assert_test_eq!("init" => "Foo instance\n");
    assert_test_eq!("class_cake" => "The German chocolate cake is delicious!\n");
    assert_test_eq!("run_class_method" => "Crunch crunch crunch!\n");
    assert_test_eq!("instance" => "Bagel instance\n");
    assert_test_eq!("class" => "DevonshireCream\n");
    assert_test_eq!("closure_scope" => "global\nglobal\n");
    assert_test_eq!("fact" => "2432902008176640000\n");
    assert_test_eq!("hidden_var" => "1\n2\n");
    assert_test_eq!("fun" => "Hi, Dear Reader!\n");
    assert_test_eq!("hello" => "Hello, world\n");
    assert_test_eq!( "env" => 
    "\
inner a
outer b
global c
outer a
outer b
global c
global a
global b
global c
");
    assert_test_eq!("fib" => 
    "\
0
1
1
1
1
2
2
3
3
5
5
8
8
13
13
21
21
34
34
55
55
89
89
144
144
233
233
377
377
610
610
987
987
1597
1597
2584
2584
4181
4181
6765
6765
10946
");

    Ok(())
}

// helpers

fn run_test_with_output(name: &str) -> Result<String> {
    let out = std::process::Command::new("cargo")
        .args(&["t", "-q", "--", "--exact", name, "--nocapture"])
        .output()?;
    Ok(parse_success(String::from_utf8(out.stdout)?))
}

fn parse_success(stdout: String) -> String {
    const START: &str = "running 1 test\n";
    const END: &str = ".\ntest result";
    let start_idx = stdout.find(START).unwrap() + START.len();
    let end_idx = stdout.find(END).unwrap();
    stdout[start_idx..end_idx].to_owned()
}
