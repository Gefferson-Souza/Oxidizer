use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_class_inheritance_basic() {
    assert_output_equivalent(
        r#"
class Animal {
    name: string;
    constructor(name: string) {
        this.name = name;
    }
    speak(): string {
        return this.name + " makes a sound";
    }
}

class Dog extends Animal {
    breed: string;
    constructor(name: string, breed: string) {
        super(name);
        this.breed = breed;
    }
    speak(): string {
        return this.name + " barks";
    }
}

function run(): void {
    const dog = new Dog("Rex", "Labrador");
    console.log(dog.speak());
    console.log(dog.breed);
}
run();
"#,
    );
}

#[test]
fn test_equivalence_class_inheritance_method_override() {
    assert_output_equivalent(
        r"
class Shape {
    area(): number {
        return 0;
    }
}

class Circle extends Shape {
    radius: number;
    constructor(radius: number) {
        super();
        this.radius = radius;
    }
    area(): number {
        return 3.14159 * this.radius * this.radius;
    }
}

function run(): void {
    const c = new Circle(5);
    console.log(c.area());
}
run();
",
    );
}
