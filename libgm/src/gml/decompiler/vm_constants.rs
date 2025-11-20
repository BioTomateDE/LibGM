//! Contains constant values used by the `GameMaker` VM.
pub mod functions {
    // Function names used for try...catch...finally statements
    pub const TRY_HOOK: &str = "@@try_hook@@";
    pub const TRY_UNHOOK: &str = "@@try_unhook@@";
    pub const FINISH_CATCH: &str = "@@finish_catch@@";
    pub const FINISH_FINALLY: &str = "@@finish_finally@@";

    // Function names for creating methods/structs
    pub const METHOD: &str = "method";
    pub const NULL_OBJECT: &str = "@@NullObject@@";
    pub const NEW_OBJECT: &str = "@@NewGMLObject@@";

    // Function name used to copy static information from an inherited constructor function in GML
    pub const COPY_STATIC: &str = "@@CopyStatic@@";

    // Function name used upon entering constructor functions (in newer versions)
    pub const SET_STATIC: &str = "@@SetStatic@@";

    // Instance type helpers used in GMLv2
    pub const SELF: &str = "@@This@@";
    pub const OTHER: &str = "@@Other@@";
    pub const GLOBAL: &str = "@@Global@@";
    pub const GET_INSTANCE: &str = "@@GetInstance@@";

    // Used to create array literals in GML
    pub const NEW_ARRAY: &str = "@@NewGMLArray@@";

    // Function name used to throw an object/exception
    pub const THROW: &str = "@@throw@@";

    // Function name used to set struct variables (used to de-optimize to be closer to source code)
    pub const STRUCT_GET_FROM_HASH: &str = "struct_get_from_hash";

    // Special-case GML functions used during macro resolution
    pub const CHOOSE: &str = "choose";
    pub const SCRIPT_EXECUTE: &str = "script_execute";

    // Function used to get static structs from functions
    pub const STATIC_GET: &str = "static_get";
}

pub mod variables {
    // Used to store return values before cleaning up stack
    pub const TEMP_RETURN: &str = "$$temp$$";

    // Variable names used by compiler to rewrite try/catch/finally
    pub const TRY_BREAK: &str = "__yy_breakEx";
    pub const TRY_CONTINUE: &str = "__yy_continueEx";
    pub const TRY_COPY: &str = "copyVar";
}

pub mod arrays {
    use std::collections::HashSet;
    use std::sync::LazyLock;

    // The size limit of arrays in GMLv1 (old GML). Used for 2D array accesses in the VM.
    pub const OLD_ARRAY_LIMIT: i32 = 32000;

    // Builtin array variables (some of which don't exist past GMS2, but are still recognized by the compiler apparently)
    pub static BUILTIN_ARRAY_VARIABLES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
        HashSet::from([
            "view_xview",
            "view_yview",
            "view_wview",
            "view_hview",
            "view_angle",
            "view_hborder",
            "view_vborder",
            "view_hspeed",
            "view_vspeed",
            "view_object",
            "view_xport",
            "view_yport",
            "view_wport",
            "view_hport",
            "view_surface_id",
            "view_camera",
            "phy_collision_x",
            "phy_collision_y",
            "phy_col_normal_x",
            "phy_col_normal_y",
        ])
    });
}
