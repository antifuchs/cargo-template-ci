macro_rules! define_matrix_entry {
    ($name:ident,
     ($run_default:expr,
      $version_default:expr,
      $install_default:expr,
      $commandline_default:expr)) => {
        #[derive(Debug)]
        pub(crate) struct $name(MatrixEntry);

        impl MatrixEntryExt for $name {
            fn the_entry<'a>(&'a self) -> &'a MatrixEntry {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                let cmdline: Option<String> = $commandline_default.into();
                $name(MatrixEntry {
                    run: $run_default,
                    version: String::from($version_default),
                    install_commandline: $install_default.into(),
                    commandline: cmdline.unwrap_or("/bin/false".to_owned()),
                })
            }
        }

        // Since we can't easily (or at all?) pass default expresisons
        // to serde, we have to define our own
        // deserializer. Thankfully, you can deserialize into an
        // intermediate struct and then assign / default the values
        // from Default::default().
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                #[derive(Deserialize)]
                struct DeserializationStruct {
                    run: Option<bool>,
                    version: Option<String>,
                    install_commandline: Option<String>,
                    commandline: Option<String>,
                }
                impl<'a> Default for DeserializationStruct {
                    fn default() -> Self {
                        DeserializationStruct {
                            run: Some($run_default),
                            version: Some(String::from($version_default)),
                            install_commandline: $install_default.into(),
                            commandline: $commandline_default.into(),
                        }
                    }
                }
                let raw: DeserializationStruct = DeserializationStruct::deserialize(deserializer)?;
                let res = $name(MatrixEntry {
                    run: raw.run.or(DeserializationStruct::default().run).unwrap(),
                    version: raw
                        .version
                        .or(DeserializationStruct::default().version)
                        .unwrap(),
                    install_commandline: raw
                        .install_commandline
                        .or(DeserializationStruct::default().install_commandline),
                    commandline: raw
                        .commandline
                        .or(DeserializationStruct::default().commandline)
                        .expect("Matrix entries need a commandline"),
                });
                Ok(res)
            }
        }
    };
}
