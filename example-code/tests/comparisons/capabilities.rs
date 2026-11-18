//! Code from `docs/comparisons/capabilities.md` — *Capabilities*.

/// ## Providers declare requirements on a context
pub mod a_requirement_on_the_context {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self, #[implicit] name: &str) -> String {
            format!("Hello, {name}!")
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub name: String,
    }

    delegate_components! {
        App {
            GreeterComponent: GreetHello,
        }
    }

    check_components! { App { GreeterComponent } }

    #[test]
    fn the_context_discharges_the_requirement() {
        let app = App {
            name: "World".to_owned(),
        };
        assert_eq!(app.greet(), "Hello, World!");
    }
}

/// ## An object capability can live in the context
pub mod an_object_capability_can_live_in_the_context {
    use cap_std::fs::Dir;
    use cgp::prelude::*;

    #[cgp_component(ConfigReader)]
    pub trait CanReadConfig {
        fn read_config(&self) -> std::io::Result<String>;
    }

    #[cgp_impl(new ReadConfigFromDir)]
    impl ConfigReader {
        fn read_config(&self, #[implicit] config_dir: &Dir) -> std::io::Result<String> {
            let mut text = String::new();
            std::io::Read::read_to_string(&mut config_dir.open("config.txt")?, &mut text)?;
            Ok(text)
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub config_dir: Dir,
    }

    delegate_components! {
        App {
            ConfigReaderComponent: ReadConfigFromDir,
        }
    }

    check_components! { App { ConfigReaderComponent } }

    #[test]
    fn the_handle_reaches_only_the_provider_that_declares_it() {
        use cap_std::ambient_authority;

        let root = std::env::temp_dir().join(format!(
            "cgp-comparisons-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("config.txt"), "port = 8080\n").unwrap();

        // Ambient authority is exercised once, and the call marks it.
        let config_dir = Dir::open_ambient_dir(&root, ambient_authority()).unwrap();
        let app = App { config_dir };

        assert_eq!(app.read_config().unwrap(), "port = 8080\n");
        // A path that would leave the directory is refused by the handle itself.
        assert!(app.config_dir.open("../secret.txt").is_err());

        std::fs::remove_dir_all(&root).unwrap();
    }
}
