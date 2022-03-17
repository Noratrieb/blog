# Types in Rust, Typescript and Java

I find programming languages very interesting, but what I find even more interesting is comparing them
and seeing their similarities and differences. So let's do that!

## Employees

Say you want to build an employee management program. You could model the employees like this:

```java
public class Employee {
    public String name;
    public int number;
}
```

> But using public fields in Java is bad practice?

Go away, I don't care.

```ts
type Employee = {
    name: string;
    number: number;
}
```

```rust
struct Employee {
    pub name: String,
    pub number: i32,
}
```

They look very similar, don't they? Sure, the syntax differs a bit and Typescript only allows floats (yikes) but they
feel like they are the same. 

Now we crate a function to print their names.

```java
public class Employee {
    public static void printName(Employee employee) {
        System.out.println(employee.name);
    }
}
```

> Why is the method static???

Because I decided so.

```ts
const printName = (employee: Employee) => {
    console.log(employee.name)
};
```

```rust
fn print_name(employee: Employee) {
    println!("{}", employee.name);
}
```


Still nothing special. Time to call it!

```java
Employee.printName(new Employee("nils", 1));
```

> Assuming we have ~~written~~ auto-generated a constructor

```ts
printName({ name: 'nils', number: 1 })
```

```rust
print_name(Employee { name: "nils".to_owned(), number: 1 })
```

> What are you doing with the `to_owned` on the string?

Rust strings are complicated.

Hmm, this is interesting. There is one big difference among the three calls: `{ name: 'nils', number: 1 }`. 
The Typescript code is not saying that its creating an `Employee`, it's just creating some object. What does that mean?

## Structural Typing

That's because Typescript uses structural typing. To quote wikipedia:
> A structural type system is a major class of type systems in which type compatibility and 
> equivalence are determined by the type's actual structure or definition and not by 
> other characteristics such as its name or place of declaration. 

Structural typing means that the type checker only looks at the structure of the types to see whether they match, instead
of looking at the names (how Java and Rust do it). 