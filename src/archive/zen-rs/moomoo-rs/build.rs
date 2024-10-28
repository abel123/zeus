use std::fs;
use std::io::Result;

fn main() -> Result<()> {
    //return Ok(());
    #[allow(unreachable_code)]
    {
        let ls = fs::read_dir("proto")?
            .map(|e| e.map(|i| i.file_name()).unwrap())
            .map(|p| format!("proto/{}", p.into_string().unwrap()))
            .collect::<Vec<String>>();

        println!("{:?}", ls,);
        let mut config = prost_build::Config::new();
        config
            .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
            .compile_well_known_types()
            .out_dir("src/pb/futu")
            .include_file("mod.rs")
            .compile_protos(&ls, &["proto/"])?;

        Ok(())
    }
}
