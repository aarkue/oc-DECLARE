use pyo3::{exceptions::PyIOError, prelude::*};
use shared::{OCDeclareDiscoveryOptions, process_mining::{self, ocel::linked_ocel::IndexLinkedOCEL}};

#[pyclass]
/// Pre-Processed OCEL
struct ProcessedOCEL {
    locel: IndexLinkedOCEL,
}


#[pyclass]
/// An individual OC-DECLARE constraint arc
struct OCDeclareArc {
    arc: shared::OCDeclareArc,
}


#[pymethods]
impl OCDeclareArc {
    /// Get string representation of OC-DECLARE arc
    pub fn to_string(&self) -> String {
        self.arc.as_template_string()
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
#[pyo3(signature = (processed_ocel: "ProcessedOCEL", noise_thresh: "double", /) -> "int")]
/// Discover OC-DECLARE constraints given a pre-processed OCEL and a noise threshold
fn discover(processed_ocel: &ProcessedOCEL,noise_thresh: f64,) -> PyResult<Vec<OCDeclareArc>> {
    let mut options = OCDeclareDiscoveryOptions::default();
    options.noise_threshold = noise_thresh;
    let discovered_constraints = shared::discover_behavior_constraints(&processed_ocel.locel, options);
    Ok(discovered_constraints.into_iter().map(|arc| OCDeclareArc { arc}).collect())
}

/// OC-DECLARE Binding for Python
#[pymodule]
fn oc_declare(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ProcessedOCEL>()?;
    m.add_class::<OCDeclareArc>()?;
    m.add_function(wrap_pyfunction!(import_ocel2, m)?)?;
    m.add_function(wrap_pyfunction!(discover, m)?)?;
    Ok(())
}
