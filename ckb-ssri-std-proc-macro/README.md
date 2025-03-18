# Molecule Schema Proc Macro

A procedural macro for generating Rust types from Molecule schema definitions.

## Usage

```rust
use ckb_ssri_std_proc_macro::mol_schema;

#[mol_schema(r#"
    array Byte32 [byte; 32];
    vector Byte32Vec <Byte32>;
    
    table Script {
        code_hash: Byte32,
        hash_type: byte,
        args: Bytes,
    }
    
    option ScriptOpt (Script);
    
    table UDTPausableData {
        pause_list: Byte32Vec,
        next_type_script: ScriptOpt,
    }
"#)]
pub struct CodecMap;

fn main() {
    // Get the singleton instance
    let codecs = CodecMap::get_instance();

    // Create and encode types
    let script = Script {
        code_hash: [0u8; 32],
        hash_type: 1,
        args: vec![],
    };
    let encoded = codecs.Script().encode(&script);
}
```

