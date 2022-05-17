use std::fs::File;
use std::io::BufReader;

use jars::{java, method, Any, ClassPat, Jar, Result};

fn main() -> Result<()> {
    let file = File::open("myjar.jar")?;
    let mut jar = Jar::new(BufReader::new(file))?;

    let class_pat = ClassPat::default()
        .public()
        .abstract_()
        .with(method!(public (String) -> ()))
        .with(method!(public static (String) -> i32))
        .with(method!(private (Any, java::Object) -> ()));
    // the pattern above would find a class looking like this:
    // public abstract class MyClass {
    //   public void m1(String arg1);
    //   public static int m2(String arg1);
    //   private void m3(* arg1, Object arg2);
    // }

    let [entry] = jars::search_exact(&mut jar, &[class_pat])?;
    let class = entry.parse()?;

    println!("our match: {}", class.this_class);

    Ok(())
}
