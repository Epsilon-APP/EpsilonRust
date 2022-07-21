use crate::controller::definitions::epsilon_instance::EpsilonInstance;
use crate::controller::definitions::epsilon_queue::EpsilonQueue;
use kube::CustomResourceExt;
use std::fs;

fn main() {
    let path_name = "./resources";

    fs::create_dir(path_name).ok();

    fs::write(
        format!("{}/{}", path_name, "epsilon_instance-definition.yaml"),
        serde_yaml::to_string(&EpsilonInstance::crd()).unwrap(),
    )
    .unwrap();

    fs::write(
        format!("{}/{}", path_name, "epsilon_queue-definition.yaml"),
        serde_yaml::to_string(&EpsilonQueue::crd()).unwrap(),
    )
    .unwrap();
}
