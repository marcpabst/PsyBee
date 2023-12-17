fn main()
{

    let mut bar = Bar { ... };
let a = bar.foo_one();
let b = bar.foo_two();
bar.set_foo_three(2);

println!("{}, {}", a, b);
}