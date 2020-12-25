use crate::address::LocalAddress;
use crate::error::{Error, Result};
use crate::model::ComponentModel;
use crate::{arraystring, Address, CompId, StringId, Var, VarType};
use fnv::FnvHashMap;
use std::collections::HashMap;

// use crate::error::Result;

// type SmallStorageIndex = (StorageIndex, VarType);
pub type StorageIndex = (StringId, StringId);

/// Main data store of the entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    map: FnvHashMap<StorageIndex, Var>,
}
impl Storage {
    pub fn new() -> Self {
        Storage {
            map: FnvHashMap::default(),
        }
    }
}
/// Generic type getters.
impl Storage {
    pub fn get_var(&self, idx: &StorageIndex, var_type: Option<VarType>) -> Option<&Var> {
        self.map.get(&idx)
    }
    pub fn get_var_mut(
        &mut self,
        idx: &StorageIndex,
        var_type: Option<VarType>,
    ) -> Option<&mut Var> {
        self.map.get_mut(&idx)
    }

    pub fn get_var_from_addr(&self, addr: &Address, comp_uid: Option<&CompId>) -> Result<&Var> {
        match comp_uid {
            Some(_comp_uid) => {
                return self
                    .map
                    .get(&(*_comp_uid, addr.var_id))
                    .ok_or(Error::FailedGettingVariable(addr.to_string()))
            }
            None => {
                return self
                    .map
                    .get(&(addr.component, addr.var_id))
                    .ok_or(Error::FailedGettingVariable(addr.to_string()))
            }
        };
    }
    pub fn get_var_mut_from_addr(
        &mut self,
        addr: &Address,
        comp_uid: Option<&CompId>,
    ) -> Result<&mut Var> {
        match comp_uid {
            Some(_comp_uid) => {
                return self
                    .map
                    .get_mut(&(*_comp_uid, addr.var_id))
                    .ok_or(Error::FailedGettingVariable(addr.to_string()))
            }
            None => {
                return self
                    .map
                    .get_mut(&(addr.component, addr.var_id))
                    .ok_or(Error::FailedGettingVariable(addr.to_string()))
            }
        };
    }
}
/// Generic type setters and inserts.
impl Storage {
    pub fn remove_comp_vars(&mut self, comp_name: &CompId, comp_model: &ComponentModel) {
        unimplemented!();
    }
    pub fn set_from_str(&mut self, target: &Address, val: &str) {
        unimplemented!();
    }
    pub fn set_from_addr(&mut self, target: &Address, source: &Address) {
        unimplemented!();
    }
    pub fn set_from_local_addr(&mut self, target: &LocalAddress, source: &LocalAddress) {
        let var = self
            .get_var(&source.storage_index().unwrap(), Some(source.var_type))
            .unwrap()
            .clone();
        let mut target = self
            .get_var_mut(&target.storage_index().unwrap(), Some(target.var_type))
            .unwrap();
        *target = var;
    }
    pub fn set_from_var(&mut self, target: &Address, comp_uid: Option<&CompId>, var: &Var) {
        let target = self
            .get_var_mut(&(target.component, target.var_id), Some(target.var_type))
            .unwrap();
        *target = var.clone();
    }

    pub fn insert(&mut self, comp_name: &str, var_id: &str, var_type: &VarType, var: Option<Var>) {
        let var_suid = (
            arraystring::new_truncate(comp_name),
            arraystring::new_truncate(var_id),
        );
        if let Some(v) = var {
            self.insert_var(comp_name, var_id, v);
        } else {
            self.map.insert(var_suid, Var::new(var_type));
        }
    }
    pub fn insert_var(&mut self, comp_name: &str, var_id: &str, var: Var) {
        let var_suid = (
            arraystring::new_truncate(comp_name),
            arraystring::new_truncate(var_id),
        );
        self.map.insert(var_suid, var);
    }
}

/// Type-strict getters.
impl Storage {
    pub fn get_str(&self, idx: &StorageIndex) -> Option<&String> {
        match self.map.get(idx)? {
            Var::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_str_mut(&mut self, idx: &StorageIndex) -> Option<&mut String> {
        match self.map.get_mut(idx)? {
            Var::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_int(&self, idx: &StorageIndex) -> Option<&crate::Int> {
        match self.map.get(idx)? {
            Var::Int(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_int_mut(&mut self, idx: &StorageIndex) -> Option<&mut crate::Int> {
        match self.map.get_mut(idx)? {
            Var::Int(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_float(&self, idx: &StorageIndex) -> Option<&crate::Float> {
        match self.map.get(idx)? {
            Var::Float(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_float_mut(&mut self, idx: &StorageIndex) -> Option<&mut crate::Float> {
        match self.map.get_mut(idx)? {
            Var::Float(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_bool(&self, idx: &StorageIndex) -> Option<&bool> {
        match self.map.get(idx)? {
            Var::Bool(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_bool_mut(&mut self, idx: &StorageIndex) -> Option<&mut bool> {
        match self.map.get_mut(idx)? {
            Var::Bool(s) => Some(s),
            _ => None,
        }
    }
}
/// Type-strict getters for lists.
impl Storage {
    pub fn get_str_list(&self, idx: &StorageIndex) -> Option<&Vec<String>> {
        match self.map.get(idx)? {
            Var::StrList(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_str_list_mut(&mut self, idx: &StorageIndex) -> Option<&mut Vec<String>> {
        match self.map.get_mut(idx)? {
            Var::StrList(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_int_list(&self, idx: &StorageIndex) -> Option<&Vec<crate::Int>> {
        match self.map.get(idx)? {
            Var::IntList(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_int_list_mut(&mut self, idx: &StorageIndex) -> Option<&mut Vec<crate::Int>> {
        match self.map.get_mut(idx)? {
            Var::IntList(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_float_list(&self, idx: &StorageIndex) -> Option<&Vec<crate::Float>> {
        match self.map.get(idx)? {
            Var::FloatList(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_float_list_mut(&mut self, idx: &StorageIndex) -> Option<&mut Vec<crate::Float>> {
        match self.map.get_mut(idx)? {
            Var::FloatList(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_bool_list(&self, idx: &StorageIndex) -> Option<&Vec<bool>> {
        match self.map.get(idx)? {
            Var::BoolList(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_bool_list_mut(&mut self, idx: &StorageIndex) -> Option<&mut Vec<bool>> {
        match self.map.get_mut(idx)? {
            Var::BoolList(s) => Some(s),
            _ => None,
        }
    }
}
/// Type-strict getters for grids.
#[cfg(feature = "grids")]
impl Storage {
    pub fn get_str_grid(&self, idx: &StorageIndex) -> Option<&Vec<Vec<String>>> {
        match self.map.get(idx)? {
            Var::StrGrid(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_str_grid_mut(&mut self, idx: &StorageIndex) -> Option<&mut Vec<Vec<String>>> {
        match self.map.get_mut(idx)? {
            Var::StrGrid(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_int_grid(&self, idx: &StorageIndex) -> Option<&Vec<Vec<crate::Int>>> {
        match self.map.get(idx)? {
            Var::IntGrid(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_int_grid_mut(&mut self, idx: &StorageIndex) -> Option<&mut Vec<Vec<crate::Int>>> {
        match self.map.get_mut(idx)? {
            Var::IntGrid(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_float_grid(&self, idx: &StorageIndex) -> Option<&Vec<Vec<crate::Float>>> {
        match self.map.get(idx)? {
            Var::FloatGrid(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_float_grid_mut(
        &mut self,
        idx: &StorageIndex,
    ) -> Option<&mut Vec<Vec<crate::Float>>> {
        match self.map.get_mut(idx)? {
            Var::FloatGrid(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_bool_grid(&self, idx: &StorageIndex) -> Option<&Vec<Vec<bool>>> {
        match self.map.get(idx)? {
            Var::BoolGrid(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_bool_grid_mut(&mut self, idx: &StorageIndex) -> Option<&mut Vec<Vec<bool>>> {
        match self.map.get_mut(idx)? {
            Var::BoolGrid(s) => Some(s),
            _ => None,
        }
    }
}

/// Handle getters.
impl Storage {
    pub fn get_all_handles(&self) -> Vec<(StringId, VarType, StringId)> {
        unimplemented!();
    }
    pub fn get_all_handles_of_type(&self, var_type: VarType) -> Vec<(StringId, VarType, StringId)> {
        unimplemented!();
    }
}
/// Type collection getters.
impl Storage {
    pub fn get_all_var(&self) -> &FnvHashMap<StorageIndex, Var> {
        &self.map
    }
    pub fn get_all_str(&self) -> Vec<(&StorageIndex, &String)> {
        self.map
            .iter()
            .filter(|(k, v)| v.is_str())
            .map(|(k, v)| (k, v.as_str().unwrap()))
            .collect()
    }
    pub fn get_all_int(&self) -> Vec<(&StorageIndex, &crate::Int)> {
        self.map
            .iter()
            .filter(|(_, v)| v.is_int())
            .map(|(k, v)| (k, v.as_int().unwrap()))
            .collect()
    }
    pub fn get_all_float(&self) -> Vec<(&StorageIndex, &crate::Float)> {
        self.map
            .iter()
            .filter(|(_, v)| v.is_float())
            .map(|(k, v)| (k, v.as_float().unwrap()))
            .collect()
    }
    pub fn get_all_bool(&self) -> Vec<(&StorageIndex, &bool)> {
        self.map
            .iter()
            .filter(|(_, v)| v.is_bool())
            .map(|(k, v)| (k, v.as_bool().unwrap()))
            .collect()
    }
    // pub fn get_all_str_list(&self) -> Vec<(&SmallStorageIndex, &Vec<String>)> {
    //     self.map
    //         .iter()
    //         .filter(|(_, v)| v.is_str_list())
    //         .map(|(k, v)| (k, v.as_str_list().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_str_list_mut(&mut self) -> Vec<(&SmallStorageIndex, &mut Vec<String>)> {
    //     self.map
    //         .iter_mut()
    //         .filter(|(_, v)| v.is_str_list())
    //         .map(|(k, v)| (k, v.as_str_list_mut().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_int_list(&self) -> Vec<(&SmallStorageIndex, &Vec<crate::Int>)> {
    //     self.map
    //         .iter()
    //         .filter(|(_, v)| v.is_int_list())
    //         .map(|(k, v)| (k, v.as_int_list().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_int_list_mut(&mut self) -> Vec<(&SmallStorageIndex, &mut Vec<crate::Int>)> {
    //     self.map
    //         .iter_mut()
    //         .filter(|(_, v)| v.is_int_list())
    //         .map(|(k, v)| (k, v.as_int_list_mut().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_float_list(&self) -> Vec<(&SmallStorageIndex, &Vec<crate::Float>)> {
    //     self.map
    //         .iter()
    //         .filter(|(_, v)| v.is_float_list())
    //         .map(|(k, v)| (k, v.as_float_list().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_float_list_mut(&mut self) -> Vec<(&SmallStorageIndex, &mut Vec<crate::Float>)> {
    //     self.map
    //         .iter_mut()
    //         .filter(|(_, v)| v.is_float_list())
    //         .map(|(k, v)| (k, v.as_float_list_mut().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_bool_list(&self) -> Vec<(&SmallStorageIndex, &Vec<bool>)> {
    //     self.map
    //         .iter()
    //         .filter(|(_, v)| v.is_bool_list())
    //         .map(|(k, v)| (k, v.as_bool_list().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_bool_list_mut(&mut self) -> Vec<(&SmallStorageIndex, &mut Vec<bool>)> {
    //     self.map
    //         .iter_mut()
    //         .filter(|(_, v)| v.is_bool_list())
    //         .map(|(k, v)| (k, v.as_bool_list_mut().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_str_grid(&self) -> Vec<(&SmallStorageIndex, &Vec<Vec<String>>)> {
    //     self.map
    //         .iter()
    //         .filter(|(_, v)| v.is_str_grid())
    //         .map(|(k, v)| (k, v.as_str_grid().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_str_grid_mut(&mut self) -> Vec<(&SmallStorageIndex, &mut Vec<Vec<String>>)> {
    //     self.map
    //         .iter_mut()
    //         .filter(|(_, v)| v.is_str_grid())
    //         .map(|(k, v)| (k, v.as_str_grid_mut().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_int_grid(&self) -> Vec<(&SmallStorageIndex, &Vec<Vec<crate::Int>>)> {
    //     self.map
    //         .iter()
    //         .filter(|(_, v)| v.is_int_grid())
    //         .map(|(k, v)| (k, v.as_int_grid().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_int_grid_mut(&mut self) -> Vec<(&SmallStorageIndex, &mut Vec<Vec<crate::Int>>)> {
    //     self.map
    //         .iter_mut()
    //         .filter(|(_, v)| v.is_int_grid())
    //         .map(|(k, v)| (k, v.as_int_grid_mut().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_float_grid(&self) -> Vec<(&SmallStorageIndex, &Vec<Vec<crate::Float>>)> {
    //     self.map
    //         .iter()
    //         .filter(|(_, v)| v.is_float_grid())
    //         .map(|(k, v)| (k, v.as_float_grid().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_float_grid_mut(&mut self) -> Vec<(&SmallStorageIndex, &mut Vec<Vec<crate::Float>>)> {
    //     self.map
    //         .iter_mut()
    //         .filter(|(_, v)| v.is_float_grid())
    //         .map(|(k, v)| (k, v.as_float_grid_mut().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_bool_grid(&self) -> Vec<(&SmallStorageIndex, &Vec<Vec<bool>>)> {
    //     self.map
    //         .iter()
    //         .filter(|(_, v)| v.is_bool_grid())
    //         .map(|(k, v)| (k, v.as_bool_grid().unwrap()))
    //         .collect()
    // }
    // pub fn get_all_bool_grid_mut(&mut self) -> Vec<(&SmallStorageIndex, &mut Vec<Vec<bool>>)> {
    //     self.map
    //         .iter_mut()
    //         .filter(|(_, v)| v.is_bool_grid())
    //         .map(|(k, v)| (k, v.as_bool_grid_mut().unwrap()))
    //         .collect()
    // }
}
/// Type collection mut getters.
// impl Storage {
//     pub fn get_all_str_mut(&mut self) -> Vec<(&StorageIndex, &mut String)> {
//         self.map
//             .iter_mut()
//             .filter(|(_, v)| v.is_str())
//             .map(|((k, _), ref mut v)| (k, v.as_str_mut().as_deref_mut().unwrap()))
//             .collect()
//     }
//     pub fn get_all_int_mut(&mut self) -> Vec<(&StorageIndex, &mut crate::Int)> {
//         self.map
//             .iter_mut()
//             .filter(|(_, v)| v.is_int())
//             .map(|((k, _), ref mut v)| (k, v.as_int_mut().unwrap()))
//             .collect()
//     }
//     pub fn get_all_float_mut(&mut self) -> Vec<(&StorageIndex, &mut crate::Float)> {
//         self.map
//             .iter_mut()
//             .filter(|(_, v)| v.is_float())
//             .map(|((k, _), ref mut v)| (k, v.as_float_mut().unwrap()))
//             .collect()
//     }
//     pub fn get_all_bool_mut(&mut self) -> Vec<(&StorageIndex, &mut bool)> {
//         self.map
//             .iter_mut()
//             .filter(|(_, v)| v.is_bool())
//             .map(|((k, _), ref mut v)| (k, v.as_bool_mut().unwrap()))
//             .collect()
//     }
// }

/// Type-strict inserts
impl Storage {
    pub fn insert_int(&mut self, idx: &StorageIndex, val: crate::Int) {
        self.map.insert(*idx, Var::Int(val));
    }
    pub fn insert_int_list(&mut self, idx: &StorageIndex, val: Vec<crate::Int>) {
        self.map.insert(*idx, Var::IntList(val));
    }
}
/// Type-strict setters and inserts
impl Storage {}

/// Conversions
impl Storage {
    pub fn get_coerce_to_string(
        &self,
        source: &Address,
        comp_uid: Option<&CompId>,
    ) -> Result<String> {
        Ok(self.get_var_from_addr(source, comp_uid)?.to_string())
    }
    pub fn get_all_coerce_to_string(&self) -> HashMap<String, String> {
        let mut out_map = HashMap::new();
        for (index, var) in &self.map {
            // println!("{:?}, {:?}", index, var);
            let (comp_name, var_name) = index;
            out_map.insert(
                format!("{}:{}:{}", comp_name, var.get_type().to_str(), var_name),
                var.to_string(),
            );
        }
        out_map
    }
}
