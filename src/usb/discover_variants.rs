use serde::{Deserialize, Serialize};

use crate::{MachineType, Volume};

macro_rules! usb_devices {
    ($(
      $name:ident(
        $machine_type:expr,
        $volume:expr,
        $vid:expr,
        $manufacturer:expr,
        $pid:expr,
        $model:expr,
        $baud:expr
      )
    ),+) => {
        /// All known USB Machines.
        #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
        #[non_exhaustive]
        pub enum UsbVariant {
            $(
                /// USB connected machine-api Machine.
                $name,
            )*
        }

        impl UsbVariant {
            /// Return the baud rate based on the make/model of the
            /// USB device.
            pub fn get_baud(&self) -> Option<u32> {
                match self {
                $(
                    Self::$name => { $baud },
                )*
                }
            }

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

            /// Return the vendor id and product id based on the device.
            pub fn get_vid_pid(&self) -> (Option<u16>, Option<u16>) {
                match self {
                $(
                    Self::$name => { ($vid, $pid) },
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

usb_devices!(
    // Generic USB based FusedDeposition 3D printer
    Generic(MachineType::FusedDeposition, None, None, None, None, None, None),
    // Prusa Research Mk3
    PrusaMk3(
        MachineType::FusedDeposition,
        Some(Volume {
            width: 250.0,
            height: 210.0,
            depth: 210.0,
        }),
        Some(0x2c99),
        Some("Prusa Research".to_owned()),
        Some(0x0002),
        Some("MK3".to_owned()),
        Some(115200)
    ) // // Prusa Research Mk4
      // PrusaMk4(
      //     MachineType::FusedDeposition,
      //     Some(Volume {
      //         width: 250.0,
      //         depth: 210.0,
      //         height: 220.0,
      //     }),
      //     Some(0x2c99),
      //     Some("Prusa Research".to_owned()),
      //     Some(0x000d),
      //     Some("MK4".to_owned()),
      //     Some(115200)
      // )
);
