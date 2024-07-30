use std::{
    env,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug)]
pub struct Target {
    pub architecture: String,
    pub vendor: String,
    pub system: String,
    pub abi: Option<String>,
}

impl Display for Target {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}-{}-{}", &self.architecture, &self.vendor, &self.system)?;

        if let Some(ref abi) = self.abi {
            write!(f, "-{abi}")
        } else {
            Result::Ok(())
        }
    }
}

impl Target {
    pub fn new() -> Target {
        let target_str = env::var("TARGET").expect("$TARGET not set");
        Target::parse_target(target_str)
    }

    pub fn parse_target(target_str: impl AsRef<str>) -> Target {
        let target_str = target_str.as_ref();
        let target: Vec<String> = target_str.split('-').map(|s| s.into()).collect();
        assert!(target.len() >= 3, "Failed to parse $TARGET {}", target_str);

        let abi = if target.len() > 3 {
            Some(target[3].clone())
        } else {
            None
        };

        Target {
            architecture: target[0].clone(),
            vendor: target[1].clone(),
            system: target[2].clone(),
            abi,
        }
    }

    pub fn is_macos(&self) -> bool {
        self.system == "darwin"
    }

    pub fn is_windows(&self) -> bool {
        self.system == "windows"
    }

    pub fn is_linux(&self) -> bool {
        self.system != "darwin" && self.system != "windows"
    }
}
