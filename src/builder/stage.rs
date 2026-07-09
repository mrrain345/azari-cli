/// A single stage in a multi-stage Containerfile build.
#[derive(Debug, Default)]
pub struct BuilderStage {
    from: Option<String>,
    early_instr: Vec<String>,
    instr: Vec<String>,
    late_instr: Vec<String>,
}

impl BuilderStage {
    /// Returns the `FROM` image reference for this stage, if any.
    pub fn from(&self) -> Option<&str> {
        self.from.as_deref()
    }

    /// Sets the `FROM` image reference for this stage.
    pub fn set_from(&mut self, from: String) {
        self.from = Some(from);
    }

    /// Appends a single Containerfile instruction line to this stage.
    pub fn push(&mut self, line: impl Into<String>) {
        self.instr.push(line.into());
    }

    /// Appends a single Containerfile instruction line to this stage,
    /// to be rendered before the main instructions.
    pub fn push_early(&mut self, line: impl Into<String>) {
        self.early_instr.push(line.into());
    }

    /// Appends a single Containerfile instruction line to this stage,
    /// to be rendered after the main instructions.
    pub fn push_late(&mut self, line: impl Into<String>) {
        self.late_instr.push(line.into());
    }

    /// Renders this stage into Containerfile text.
    ///
    /// `name` is the stage name used in the `FROM … AS {name}` header.
    /// If `from` is `None` the header line is omitted entirely.
    pub fn to_containerfile(&self, name: &str) -> String {
        let mut lines: Vec<&str> = Vec::new();

        let header;
        if let Some(from) = &self.from {
            header = format!("FROM {from} AS {name}");
            lines.push(&header);
        }

        for s in &self.early_instr {
            lines.push(s.as_str());
        }
        for s in &self.instr {
            lines.push(s.as_str());
        }
        for s in &self.late_instr {
            lines.push(s.as_str());
        }

        if lines.is_empty() {
            return String::new();
        }

        lines.push(""); // trailing newline
        lines.join("\n")
    }
}
