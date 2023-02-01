use anyhow::Result;
use j4rs::{Instance, InvocationArg, Jvm, JvmBuilder};

fn main() -> Result<()> {
    println!("Hello, world!");

    let jvm = JvmBuilder::new().build()?;

    let string_instance = jvm.create_instance(
        "java.lang.String", // The Java class to create an instance for
        &Vec::new(), // The `InvocationArg`s to use for the constructor call - empty for this example
    )?;
    let boolean_instance = jvm.invoke(
        &string_instance, // The String instance created above
        "isEmpty",        // The method of the String instance to invoke
        &Vec::new(),      // The `InvocationArg`s to use for the invocation - empty for this example
    )?;
    let rust_boolean: bool = jvm.to_rust(boolean_instance)?;
    println!(
        "The isEmpty() method of the java.lang.String instance returned {}",
        rust_boolean
    );
    let _static_invocation_result = jvm.invoke_static(
        "java.lang.System",  // The Java class to invoke
        "currentTimeMillis", // The static method of the Java class to invoke
        &Vec::new(), // The `InvocationArg`s to use for the invocation - empty for this example
    )?;
    let system_class = jvm.static_class("java.lang.System")?;
    let system_out_field = jvm.field(&system_class, "out");
    let access_mode_enum = jvm.static_class("java.nio.file.AccessMode")?;
    let access_mode_write = jvm.field(&access_mode_enum, "WRITE")?;
    Ok(())
}
