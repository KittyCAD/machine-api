use crate::{MachineType, Volume};
use serde::{Deserialize, Serialize};

macro_rules! moonraker_devices {
    ($(
      $name:ident(
        $machine_type:expr,
        $volume:expr,
        $manufacturer:expr,
        $model:expr
      )
    ),+) => {
        /// All known Moonraker based Machines.
        #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
        pub enum MoonrakerVariant {
            $(
                /// Moonraker connected machine-api Machine.
                $name,
            )*
        }

        impl MoonrakerVariant {
            /// Return the max manufacture volume.
            pub fn get_max_part_volume(&self) -> Option<Volume> {
                match self {
                $(
                    Self::$name => { $volume }
                )*
                }
            }

            /// Return the machine's method of manufacture
            pub fn get_machine_type(&self) -> MachineType {
                match self {
                $(
                    Self::$name => { $machine_type }
                )*
                }
            }

            /// Return the make/model of the device.
            pub fn get_manufacturer_model(&self) -> (Option<String>, Option<String>) {
                match self {
                $(
                    Self::$name => { ($manufacturer, $model) },
                )*
                }
            }
        }
    };
}

moonraker_devices!(
    // Generic Moonraker based FusedDeposition 3D printer
    Generic(MachineType::FusedDeposition, None, None, None),
    // Elegoo Neptune 4
    ElegooNeptune4(
        MachineType::FusedDeposition,
        Some(Volume {
            width: 250.0,
            height: 250.0,
            depth: 250.0,
        }),
        Some("Elegoo".to_owned()),
        Some("Neptune 4".to_owned())
    )
);
