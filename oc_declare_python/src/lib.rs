use pyo3::{
    exceptions::{PyIOError, PyValueError},
    prelude::*,
};
use shared::{
    process_mining::{self, ocel::linked_ocel::IndexLinkedOCEL},
    O2OMode, OCDeclareArcLabel, OCDeclareArcType, OCDeclareDiscoveryOptions, OCDeclareNode,
    ObjectTypeAssociation,
};

#[pyclass]
/// Pre-Processed OCEL
struct ProcessedOCEL {
    locel: IndexLinkedOCEL,
}

#[derive(Debug, Clone)]
#[pyclass]
/// An individual OC-DECLARE constraint arc
struct OCDeclareArc {
    arc: shared::OCDeclareArc,
}

#[pymethods]
impl OCDeclareArc {
    #[new]
    /// Construct a new OC-DECLARE arc
    ///
    #[pyo3(signature = (from_act: "str", to_act: "str", arc_type: "Literal['AS', 'EF', 'EP', 'DF', 'DP']", min_count: "Optional[int]", max_count: "Optional[int]", /, all_ots: "list[str]"= vec![], each_ots: "list[str]"= vec![], any_ots: "list[str]"= vec![]) -> "OCDeclareArc")]
    pub fn new(
        from_act: String,
        to_act: String,
        arc_type: String,
        min_count: Option<usize>,
        max_count: Option<usize>,
        all_ots: Vec<String>,
        each_ots: Vec<String>,
        any_ots: Vec<String>,
    ) -> PyResult<Self> {
        let arc_type = OCDeclareArcType::parse_str(&arc_type)
            .ok_or(PyErr::new::<PyValueError, _>("Invalid arc type."))?;
        let label = OCDeclareArcLabel {
            each: each_ots
                .into_iter()
                .map(|ot| ObjectTypeAssociation::Simple { object_type: ot })
                .collect(),
            any: any_ots
                .into_iter()
                .map(|ot| ObjectTypeAssociation::Simple { object_type: ot })
                .collect(),
            all: all_ots
                .into_iter()
                .map(|ot| ObjectTypeAssociation::Simple { object_type: ot })
                .collect(),
        };
        let arc = shared::OCDeclareArc {
            from: OCDeclareNode::new(from_act),
            to: OCDeclareNode::new(to_act),
            arc_type,
            label,
            counts: (min_count, max_count),
        };
        Ok(Self { arc: arc })
    }
    /// Get string representation of OC-DECLARE arc
    pub fn to_string(&self) -> String {
        self.arc.as_template_string()
    }

    pub fn __repr__(&self) -> String {
        format!("OC-DECLARE Arc: {}", self.to_string())
    }

    pub fn __str__(&self) -> String {
        self.to_string()
    }

    // Write getters/setters for all fields of OCDeclareArc
    // Add documentation to each
    /// Get the source activity of the arc.
    #[getter]
    pub fn from_activity(&self) -> String {
        self.arc.from.as_str().to_string()
    }

    /// Get the target activity of the arc.
    #[getter]
    pub fn to_activity(&self) -> String {
        self.arc.to.as_str().to_string()
    }

    /// Get the type of the arc (e.g., "EF", "DF", "AS").
    #[getter]
    pub fn arc_type_name(&self) -> String {
        self.arc.arc_type.get_name().to_string()
    }

    /// Get the object types involved with the 'ALL' quantifier.
    #[getter]
    pub fn all_ots(&self) -> Vec<String> {
        self.arc
            .label
            .all
            .iter()
            .map(|ota| ota.as_template_string())
            .collect()
    }

    /// Get the object types involved with the 'EACH' quantifier.
    #[getter]
    pub fn each_ots(&self) -> Vec<String> {
        self.arc
            .label
            .each
            .iter()
            .map(|ota| ota.as_template_string())
            .collect()
    }

    /// Get the object types involved with the 'ANY' quantifier.
    #[getter]
    pub fn any_ots(&self) -> Vec<String> {
        self.arc
            .label
            .any
            .iter()
            .map(|ota| ota.as_template_string())
            .collect()
    }

    /// Get the minimum count for the arc.
    #[getter]
    pub fn min_count(&self) -> Option<usize> {
        self.arc.counts.0
    }

    /// Get the maximum count for the arc.
    #[getter]
    pub fn max_count(&self) -> Option<usize> {
        self.arc.counts.1
    }

    // Next, setters
    // Again, add documentation
    /// Set the source activity of the arc.
    #[setter]
    pub fn set_from_activity(&mut self, from_act: String) {
        self.arc.from = OCDeclareNode::new(from_act);
    }

    /// Set the target activity of the arc.
    #[setter]
    pub fn set_to_activity(&mut self, to_act: String) {
        self.arc.to = OCDeclareNode::new(to_act);
    }

    /// Set the type of the arc (e.g., "EF", "DF", "AS").
    #[setter]
    pub fn set_arc_type(&mut self, arc_type: String) -> PyResult<()> {
        self.arc.arc_type =
            OCDeclareArcType::parse_str(&arc_type)
                .ok_or(PyErr::new::<PyValueError, _>("Invalid arc type."))?;
        Ok(())
    }

    /// Set the object types involved with the 'ALL' quantifier.
    #[setter]
    pub fn set_all_ots(&mut self, all_ots: Vec<String>) {
        self.arc.label.all = all_ots
            .into_iter()
            .map(|ot| ObjectTypeAssociation::Simple { object_type: ot })
            .collect();
    }

    /// Set the object types involved with the 'EACH' quantifier.
    #[setter]
    pub fn set_each_ots(&mut self, each_ots: Vec<String>) {
        self.arc.label.each = each_ots
            .into_iter()
            .map(|ot| ObjectTypeAssociation::Simple { object_type: ot })
            .collect();
    }

    /// Set the object types involved with the 'ANY' quantifier.
    #[setter]
    pub fn set_any_ots(&mut self, any_ots: Vec<String>) {
        self.arc.label.any = any_ots
            .into_iter()
            .map(|ot| ObjectTypeAssociation::Simple { object_type: ot })
            .collect();
    }

    /// Set the minimum count for the arc.
    #[setter]
    pub fn set_min_count(&mut self, min_count: Option<usize>) {
        self.arc.counts.0 = min_count;
    }

    /// Set the maximum count for the arc.
    #[setter]
    pub fn set_max_count(&mut self, max_count: Option<usize>) {
        self.arc.counts.1 = max_count;
    }
}

#[pyfunction]
#[pyo3(signature = (path: "str", /) -> "ProcessedOCEL")]
/// Import an OCEL 2.0 file (.xml or .json) and preprocess it for use with OC-DECLARE
fn import_ocel2(path: String) -> PyResult<ProcessedOCEL> {
    let ocel = if path.ends_with(".xml") {
        process_mining::import_ocel_xml_file(path)
    } else if path.ends_with(".json") {
        process_mining::import_ocel_json_from_path(path)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
    } else {
        return Err(PyErr::new::<PyIOError, _>(
            "Invalid format! Currently only .json and .xml files are supported.",
        ));
    };
    let locel = shared::preprocess_ocel(ocel);
    Ok(ProcessedOCEL { locel })
}

#[pyfunction]
#[pyo3(signature = (processed_ocel: "ProcessedOCEL", /, noise_thresh: "double" = 0.2, acts_to_use: "Optional[list[str]]" = None, o2o_mode: "Optional[Literal['None', 'Direct', 'Reversed', 'Bidirectional']]"  = None) -> "list[OCDeclareArc]")]
/// Discover OC-DECLARE constraints given a pre-processed OCEL and a noise threshold
fn discover(
    processed_ocel: &ProcessedOCEL,
    noise_thresh: f64,
    acts_to_use: Option<Vec<String>>,
    o2o_mode: Option<String>,
) -> PyResult<Vec<OCDeclareArc>> {
    let mut options = OCDeclareDiscoveryOptions::default();
    options.noise_threshold = noise_thresh;
    options.acts_to_use = acts_to_use;
    if let Some(o2o_mode) = o2o_mode {
        options.o2o_mode = match o2o_mode.as_str() {
            "None" => O2OMode::None,
            "Direct" => O2OMode::Direct,
            "Reversed" => O2OMode::Reversed,
            "Bidirectional" => O2OMode::Bidirectional,
            _ => return Err(PyErr::new::<PyValueError, _>("Invalid O2O mode. Valid options are: 'None', 'Direct', 'Reversed', 'Bidirectional'.")),
        };
    }
    let discovered_constraints =
        shared::discover_behavior_constraints(&processed_ocel.locel, options);
    Ok(discovered_constraints
        .into_iter()
        .map(|arc| OCDeclareArc { arc })
        .collect())
}

#[pyfunction]
#[pyo3(signature = (processed_ocel: "ProcessedOCEL", constraint: "OCDeclareArc", /) -> "double")]
/// Evaluate an OC-DECLARE constraint given a pre-processed OCEL
/// yielding the fraction of relevant event satisfying the constraints
///
/// Returns 1 if all source events fulfill the constraint and 0 if all source events violate the constraint.
fn check_conformance(processed_ocel: &ProcessedOCEL, constraint: OCDeclareArc) -> PyResult<f64> {
    return Ok(1.0 - constraint.arc.get_for_all_evs_perf(&processed_ocel.locel));
}

/// OC-DECLARE Binding for Python
#[pymodule]
fn oc_declare(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ProcessedOCEL>()?;
    m.add_class::<OCDeclareArc>()?;
    m.add_function(wrap_pyfunction!(import_ocel2, m)?)?;
    m.add_function(wrap_pyfunction!(discover, m)?)?;
    m.add_function(wrap_pyfunction!(check_conformance, m)?)?;
    Ok(())
}
