use zed_extension_api as zed;

struct XyBuildExtension;

impl zed::Extension for XyBuildExtension {
    fn new() -> Self {
        XyBuildExtension
    }
}

zed::register_extension!(XyBuildExtension);
