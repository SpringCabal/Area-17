use spring_addons::register_gadgets;
use spring_addons::rules::GadgetHandler;

use super::*;
use crate::state::Global;

pub fn setup(handler: &mut GadgetHandler<Global>) {
    register_gadgets!(handler, [GameLoadGadget::new()]);

    #[cfg(feature = "headless-integration")]
    register_gadgets!(handler, [DbgIntegrationGadget::new()]);
    #[cfg(feature = "e2e")]
    register_gadgets!(handler, [DbgVisualIntegrationGadget::new()]);

    register_gadgets!(
        handler,
        [
            UnitFallDamageGadget::new(),
            UnitUfoControllerGadget::new(),
            UnitCivilianSpawnerGadget::new(),
            UnitDronesGadget::new(),
            UnitUfoAbductionGadget::new(),
            WeaponHandlersGadget::new(),
            UpgradesGadget::new(),
            GameTerrainGadget::new(),
            UnitSetupGadget::new(),
            UnitScriptsGadget::new(),
            ApiStatusGadget::new(),
            UnitBuildingsGadget::new(),
        ]
    );

    #[cfg(feature = "diagnostics")]
    register_gadgets!(handler, [DbgDiagnosticsGadget::new()]);
}
