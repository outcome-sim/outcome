//! Command definitions.
//!
//! Command struct serves as the basic building block for the in-memory logic
//! representation used by the runtime. Each command provides an implementation
//! for it's interpretation (converting a command prototype into target struct)
//! and for it's execution (performing work, usually work on some piece of
//! data).
//!
//! Individual *component runtimes*, as seen declared within a model, exist
//! as collections of command structs. During logic execution, these
//! collections are iterated on, executing the commands one by one.
//!
//! Command structs are stored on component models, making each component of
//! certain type contain the same set of commands.

#![allow(unused)]

extern crate getopts;
extern crate strsim;

use std::collections::{BTreeMap, HashMap};
use std::env::args;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use arrayvec::ArrayVec;
use fnv::FnvHashMap;
// use serde_yaml::Value;
use smallvec::SmallVec;

#[cfg(feature = "machine_dynlib")]
use libloading::Library;

use crate::{model, string, util, CompName, EntityId, ShortString};
use crate::{EntityName, Sim, StringId, VarType};

use crate::address::{Address, ShortLocalAddress};
use crate::entity::{Entity, EntityNonSer, Storage};
// use crate::error::Error;
use crate::model::SimModel;
// use crate::Result;
use crate::Var;

pub mod register;
// pub mod equal;
pub mod eval;
pub mod flow;
pub mod get_set;

#[cfg(feature = "machine_dynlib")]
pub mod lib;
#[cfg(feature = "machine_dynlib")]
use crate::machine::cmd::lib::LibCall;

#[cfg(feature = "machine_lua")]
pub mod lua;

pub mod print;
pub mod range;
pub mod set;
pub mod sim;

// use self::equal::*;
// use self::eval::*;
use self::get_set::*;
// use self::lib::*;
// use self::lua::*;

use crate::distr::DistributionPolicy;
use crate::distr::{CentralCommunication, SimCentral};
use crate::machine::cmd::CommandResult::JumpToLine;
use crate::machine::error::{Error, ErrorKind, Result};
use crate::machine::{CommandPrototype, CommandResultVec, ExecutionContext, LocationInfo};

// pub type CommandResult = std::result::Result<CommandOutcome, Error>;

/// Used for controlling the flow of execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResult {
    /// Continue execution
    Continue,
    /// Break execution
    Break,
    /// Jump to line
    JumpToLine(usize),
    /// Jump to tag
    JumpToTag(StringId),
    /// Execute command that needs access to another entity
    ExecExt(ExtCommand),
    /// Execute command that needs access to central authority
    ExecCentralExt(CentralRemoteCommand),
    /// Signalize that an error has occurred during execution of command
    Err(Error),
}

impl CommandResult {
    pub fn from_str(s: &str) -> Option<CommandResult> {
        if s.starts_with("jump.") {
            let c = &s[5..];
            return Some(CommandResult::JumpToLine(c.parse::<usize>().unwrap()));
        }
        //else if s.starts_with("goto.") {
        //let c = &s[5..];
        //return Some(CommandResult::GoToState(SmallString::from_str(c).unwrap()));
        //}
        else {
            match s {
                "ok" | "Ok" | "OK" | "continue" => Some(CommandResult::Continue),
                "break" => Some(CommandResult::Break),
                _ => None,
            }
        }
    }
}

/// Defines all the local commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Sim(sim::SimControl),
    Print(print::Print),
    PrintFmt(print::PrintFmt),

    Set(set::Set),
    SetIntIntAddr(set::SetIntIntAddr),

    Eval(eval::Eval),
    // EvalReg(EvalReg),

    // Equal(Equal),
    // BiggerThan(BiggerThan),
    #[cfg(feature = "machine_lua")]
    LuaScript(lua::LuaScript),
    #[cfg(feature = "machine_lua")]
    LuaCall(lua::LuaCall),
    #[cfg(feature = "machine_dynlib")]
    LibCall(lib::LibCall),

    Attach(Attach),
    Detach(Detach),
    Goto(Goto),
    Jump(Jump),

    // ext
    Get(Get),

    // central ext
    Invoke(Invoke),
    Spawn(Spawn),

    // register
    RegisterEvent(register::RegisterEvent),
    RegisterEntityPrefab(register::RegisterEntityPrefab),
    RegisterComponent(register::RegisterComponent),
    RegisterTrigger(register::RegisterTrigger),
    RegisterVar(register::RegisterVar),
    Extend(register::Extend),

    // register blocks
    State(flow::state::State),
    Component(flow::component::ComponentBlock),

    // flow control
    If(flow::ifelse::If),
    Else(flow::ifelse::Else),
    End(flow::end::End),
    Call(flow::call::Call),
    ForIn(flow::forin::ForIn),
    Loop(flow::_loop::Loop),
    Break(flow::_loop::Break),
    Procedure(flow::procedure::Procedure),

    Range(range::Range),
}

impl Command {
    /// Creates new command struct from a prototype.
    pub fn from_prototype(
        proto: &CommandPrototype,
        location: &LocationInfo,
        commands: &Vec<CommandPrototype>,
    ) -> Result<Command> {
        let cmd_name = match &proto.name {
            Some(c) => c,
            None => return Err(Error::new(location.clone(), ErrorKind::NoCommandPresent)),
        };
        let args = match &proto.arguments {
            Some(a) => a.clone(),
            None => Vec::new(),
        };
        match cmd_name.as_str() {
            "print" => Ok(Command::PrintFmt(print::PrintFmt::new(args)?)),
            "set" => Ok(set::Set::new(args, location)?),
            // "set" => Ok(get::Get::new(args, location)?),
            "spawn" => Ok(Command::Spawn(Spawn::new(args, location)?)),
            "invoke" => Ok(Command::Invoke(Invoke::new(args)?)),
            "sim" => Ok(sim::SimControl::new(args)?),

            "extend" => Ok(Command::Extend(register::Extend::new(args, location)?)),

            // register one-liners
            "event" => Ok(Command::RegisterEvent(register::RegisterEvent::new(
                args, location,
            )?)),
            "entity" | "prefab" => Ok(Command::RegisterEntityPrefab(
                register::RegisterEntityPrefab::new(args, location)?,
            )),
            "trigger" | "triggered_by" => Ok(Command::RegisterTrigger(
                register::RegisterTrigger::new(args, location)?,
            )),
            "var" => Ok(Command::RegisterVar(register::RegisterVar::new(
                args, location,
            )?)),

            "component" | "comp" => {
                Ok(register::RegisterComponent::new(args, location, &commands)?)
            }

            // register blocks
            // "component" | "comp" => Ok(flow::component::ComponentBlock::new(
            //     args, location, &commands,
            // )?),
            "state" => Ok(flow::state::State::new(args, location, &commands)?),
            "goto" => Ok(Command::Goto(Goto::new(args)?)),

            // flow control
            "jump" => Ok(Command::Jump(Jump::new(args)?)),
            "if" => Ok(Command::If(flow::ifelse::If::new(
                args, location, &commands,
            )?)),
            "else" => Ok(Command::Else(flow::ifelse::Else::new(args)?)),
            "proc" | "procedure" => Ok(Command::Procedure(flow::procedure::Procedure::new(
                args, location, &commands,
            )?)),
            "call" => Ok(Command::Call(flow::call::Call::new(
                args, location, &commands,
            )?)),
            "end" => Ok(Command::End(flow::end::End::new(args)?)),
            "for" => Ok(Command::ForIn(flow::forin::ForIn::new(
                args, location, commands,
            )?)),
            "loop" | "while" => Ok(Command::Loop(flow::_loop::Loop::new(
                args, location, commands,
            )?)),
            "break" => Ok(Command::Break(flow::_loop::Break {})),

            "range" => Ok(Command::Range(range::Range::new(args)?)),

            "eval" => Ok(eval::Eval::new(args)?),

            #[cfg(feature = "machine_dynlib")]
            "lib_call" => Ok(LibCall::new(args)?),

            _ => Err(Error::new(
                location.clone(),
                ErrorKind::UnknownCommand(cmd_name.to_string()),
            )),
        }
    }

    /// Execute `loc` phase command (within the context of single entity).
    pub fn execute(
        &self,
        ent_storage: &mut Storage,
        ent_insta: &mut EntityNonSer,
        comp_state: &mut StringId,
        call_stack: &mut super::CallStackVec,
        registry: &mut super::Registry,
        comp_name: &CompName,
        ent_id: &EntityId,
        sim_model: &SimModel,
        location: &LocationInfo,
        #[cfg(feature = "machine_dynlib")] libs: &super::Libraries,
    ) -> CommandResultVec {
        let line = location.line.unwrap();
        let mut out_res = CommandResultVec::new();
        match self {
            Command::Sim(cmd) => {
                out_res.push(cmd.execute_loc(ent_storage, comp_state, comp_name, location))
            }
            Command::Print(cmd) => out_res.push(cmd.execute_loc(ent_storage)),
            Command::PrintFmt(cmd) => {
                out_res.push(cmd.execute_loc(ent_storage, comp_state, comp_name, location))
            }
            Command::Set(cmd) => {
                out_res.push(cmd.execute_loc(ent_storage, ent_id, comp_state, comp_name, location))
            }
            Command::SetIntIntAddr(cmd) => {
                out_res.push(cmd.execute_loc(ent_storage, comp_name, location))
            }

            Command::Eval(cmd) => {
                out_res.push(cmd.execute_loc(ent_storage, comp_name, registry, location))
            }
            // Command::EvalReg(cmd) => out_res.push(cmd.execute_loc(registry)),

            //Command::Eval(cmd) => out_res.push(cmd.execute_loc(ent_storage)),
            //Command::Equal(cmd) => out_res.push(cmd.execute_loc(ent_storage)),
            //Command::BiggerThan(cmd) => out_res.push(cmd.execute_loc(ent_storage)),

            //// Command::LuaScript(cmd) => out_res.push(cmd.execute_loc(ent_storage, comp_uid)),
            //Command::LuaCall(cmd) => out_res.extend(cmd.execute_loc_lua(sim_model, ent)),
            #[cfg(feature = "machine_dynlib")]
            Command::LibCall(cmd) => out_res.push(cmd.execute_loc(libs, ent_id, ent_storage)),
            //Command::Attach(cmd) => out_res.push(cmd.execute_loc(ent, sim_model)),
            //Command::Detach(cmd) => out_res.push(cmd.execute_loc(ent, sim_model)),
            Command::Goto(cmd) => out_res.push(cmd.execute_loc(comp_state)),
            //Command::Jump(cmd) => out_res.push(cmd.execute_loc()),

            //Command::Get(cmd) => out_res.push(cmd.execute_loc()),
            Command::RegisterEntityPrefab(cmd) => out_res.extend(cmd.execute_loc()),

            Command::RegisterComponent(cmd) => out_res.extend(cmd.execute_loc(call_stack)),
            Command::RegisterVar(cmd) => out_res.extend(cmd.execute_loc(call_stack)),
            Command::RegisterTrigger(cmd) => out_res.extend(cmd.execute_loc(call_stack)),
            Command::RegisterEvent(cmd) => out_res.extend(cmd.execute_loc()),

            Command::Invoke(cmd) => out_res.push(cmd.execute_loc()),
            Command::Spawn(cmd) => out_res.push(cmd.execute_loc()),
            Command::Call(cmd) => {
                out_res.push(cmd.execute_loc(call_stack, line, sim_model, comp_name, location))
            }

            Command::Jump(cmd) => out_res.push(cmd.execute_loc()),
            Command::If(cmd) => {
                out_res.push(cmd.execute_loc(call_stack, ent_storage, comp_name, line))
            }
            Command::Else(cmd) => out_res.push(cmd.execute_loc(call_stack, ent_storage, location)),
            Command::ForIn(cmd) => out_res.push(cmd.execute_loc(
                call_stack,
                registry,
                comp_name,
                ent_storage,
                location,
            )),
            Command::Loop(cmd) => out_res.push(cmd.execute_loc(call_stack, ent_storage, line)),
            Command::Break(cmd) => out_res.push(cmd.execute_loc(call_stack, ent_storage, location)),

            Command::End(cmd) => {
                out_res.push(cmd.execute_loc(call_stack, comp_name, ent_storage, location))
            }
            Command::Procedure(cmd) => out_res.push(cmd.execute_loc(call_stack, ent_storage, line)),

            Command::State(cmd) => {
                out_res.extend(cmd.execute_loc(call_stack, ent_id, comp_name, line))
            }
            Command::Component(cmd) => {
                out_res.extend(cmd.execute_loc(call_stack, ent_id, comp_name, line))
            }

            Command::Extend(cmd) => out_res.push(cmd.execute_loc()),
            // Command::Register(cmd) => out_res.extend(cmd.execute_loc(call_stack)),
            Command::Range(cmd) => out_res.push(cmd.execute_loc(ent_storage, comp_name, location)),

            _ => out_res.push(CommandResult::Continue),
        };
        out_res
    }
    pub fn run_with_model_context(&self, sim_model: &mut SimModel) -> CommandResult {
        match self {
            // Command::Register(cmd) => cmd.execute(sim_model),
            // Command::Print(cmd) => cmd.run(),
            _ => CommandResult::Continue,
        }
    }
}

/// External (non-entity-local) command meant for execution within central
/// authority scope.
///
/// ### Distinction between remote and central-remote
///
/// Distinction is made because of the potentially distributed nature of the
/// simulation. In a distributed setting, there are certain things, like the
/// simulation model, that have to be managed from a central point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CentralRemoteCommand {
    Sim(sim::SimControl),
    // Register(register::Register),
    RegisterComponent(register::RegisterComponent),
    RegisterTrigger(register::RegisterTrigger),
    RegisterVar(register::RegisterVar),
    RegisterEntityPrefab(register::RegisterEntityPrefab),
    RegisterEvent(register::RegisterEvent),

    Extend(register::Extend),
    Invoke(Invoke),
    Spawn(Spawn),

    State(flow::state::State),
    Component(flow::component::ComponentBlock),
}
impl CentralRemoteCommand {
    /// Executes the command locally, using a reference to the monolithic `Sim`
    /// struct.
    pub fn execute(
        &self,
        mut sim: &mut Sim,
        ent_uid: &EntityId,
        comp_uid: &CompName,
    ) -> Result<()> {
        match self {
            CentralRemoteCommand::Sim(cmd) => cmd.execute_ext(sim),

            CentralRemoteCommand::RegisterComponent(cmd) => cmd.execute_ext(sim, ent_uid, comp_uid),
            CentralRemoteCommand::RegisterEntityPrefab(cmd) => cmd.execute_ext(sim),

            CentralRemoteCommand::RegisterEvent(cmd) => cmd.execute_ext(sim),
            CentralRemoteCommand::RegisterTrigger(cmd) => cmd.execute_ext(sim, ent_uid, comp_uid),
            CentralRemoteCommand::RegisterVar(cmd) => cmd.execute_ext(sim, ent_uid, comp_uid),

            CentralRemoteCommand::Extend(cmd) => cmd.execute_ext(sim, ent_uid),
            CentralRemoteCommand::Invoke(cmd) => cmd.execute_ext(sim),
            CentralRemoteCommand::Spawn(cmd) => cmd.execute_ext(sim, ent_uid),
            // CentralRemoteCommand::Prefab(cmd) => return cmd.execute_ext(sim),
            CentralRemoteCommand::State(cmd) => cmd.execute_ext(sim),
            CentralRemoteCommand::Component(cmd) => cmd.execute_ext(sim),

            _ => return Ok(()),
        }
    }
    pub fn execute_distr(
        &self,
        mut central: &mut SimCentral,
        ent_uid: &EntityId,
        comp_name: &CompName,
    ) -> Result<()> {
        match self {
            CentralRemoteCommand::Spawn(cmd) => cmd.execute_ext_distr(central)?,
            CentralRemoteCommand::RegisterEntityPrefab(cmd) => cmd.execute_ext_distr(central)?,
            CentralRemoteCommand::RegisterComponent(cmd) => cmd.execute_ext_distr(central)?,
            CentralRemoteCommand::RegisterVar(cmd) => cmd.execute_ext_distr(central, comp_name)?,
            CentralRemoteCommand::RegisterTrigger(cmd) => cmd.execute_ext_distr(central)?,
            CentralRemoteCommand::RegisterEvent(cmd) => cmd.execute_ext_distr(central)?,
            CentralRemoteCommand::State(cmd) => cmd.execute_ext_distr(central)?,
            CentralRemoteCommand::Component(cmd) => cmd.execute_ext_distr(central)?,
            CentralRemoteCommand::Invoke(cmd) => cmd.execute_ext_distr(central)?,
            _ => error!("unimplemented: {:?}", self),
        }
        Ok(())
    }
}

/// External command meant for execution on an entity scope
/// that includes operations that don't require access to
/// central authority.
///
/// Can be used to allow message-based communication between
/// entity objects.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExtCommand {
    Get(Get),
    Set(ExtSet),
    SetVar(ExtSetVar),
    // RemoteExec(Command),
    // CentralizedExec(CentralExtCommand),
}
impl ExtCommand {
    pub fn execute(
        &self,
        mut sim: &mut Sim,
        ent_id: &EntityId,
        comp_name: &CompName,
        location: &LocationInfo,
    ) -> Result<()> {
        match self {
            // ExtCommand::Get(cmd) => return cmd.execute_ext(sim, ent_uid, comp_uid, location),
            ExtCommand::Set(cmd) => return cmd.execute_ext(sim, ent_id, comp_name, location),
            // ExtCommand::SetVar(cmd) => return cmd.execute_ext(sim, exec_ctx),
            _ => return Ok(()),
        }
    }

    pub fn execute_pre(
        &self,
        mut storage: &mut Storage,
        ent_uid: &EntityName,
    ) -> Option<(Address, Var)> {
        match self {
            ExtCommand::Get(cmd) => return cmd.exec_pre(storage, ent_uid),
            _ => None,
        }
    }

    pub fn get_type_as_str(&self) -> &str {
        match self {
            // ExtCommand::Set(_) => "set",
            _ => "not implemented",
        }
    }
}

/// Attach
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "stack_stringid", derive(Copy))]
pub struct Attach {
    pub model_type: StringId,
    pub model_id: StringId,
    pub new_id: StringId,
}
impl Attach {
    // fn from_str(args_str: &str) -> MachineResult<Self> {
    //     let split: Vec<&str> = args_str.split(" ").collect();
    //     Ok(Attach {
    //         model_type: StringIndex::from(split[0]).unwrap(),
    //         model_id: StringIndex::from(split[1]).unwrap(),
    //         new_id: StringIndex::from(split[2]).unwrap(),
    //     })
    // }
    pub fn execute_loc(&self, ent: &mut Entity, sim_model: &SimModel) -> CommandResult {
        unimplemented!();
        // ent.components.attach(
        //     sim_model,
        //     &mut ent.storage,
        //     self.model_type.as_str(),
        //     self.model_id.as_str(),
        //     self.new_id.as_str(),
        // );
        CommandResult::Continue
    }
}
/// Detach
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "stack_stringid", derive(Copy))]
pub struct Detach {
    pub signature: Address,
    /* pub comp_model_type: SmallString,
     * pub comp_model_id: Option<SmallString>,
     * pub comp_id: SmallString, */
}
impl Detach {
    // TODO develop further
    fn from_str(args_str: &str) -> Result<Self> {
        let split: Vec<&str> = args_str.split(" ").collect();
        let signature = Address::from_str(split[0]).unwrap();
        Ok(Detach { signature })
        // if split.len() == 2 {
        // Ok(Detach {
        // comp_model_type:
        // SmallString::from_str_truncate(split[0]),
        // comp_id: ArrSSmallStringtr10::from_str_truncate(split[1]),
        // comp_model_id: None,
        //})
        //} else if split.len() == 3 {
        // Ok(Detach {
        // comp_model_type:
        // SmallString::from_str_truncate(split[0]),
        // comp_id: SmallString::from_str_truncate(split[1]),
        // comp_model_id:
        // Some(SmallString::from_str_truncate(split[2])),
        //})
        //} else {
        // Err(format!("wrong number of args"))
        //}
    }
    pub fn execute_loc(&self, ent: &mut Entity, sim_model: &SimModel) -> CommandResult {
        unimplemented!();
        // let comp_model = sim_model
        //     .get_component(
        //         &ent.model_type,
        //         &self.signature.get_comp_type_safe().unwrap(),
        //         &self.signature.get_comp_id_safe().unwrap(),
        //     )
        //     .unwrap();
        // ent.components.detach(
        //     &mut ent.storage,
        //     &comp_model,
        //     //&self.signature.comp_type.unwrap(),
        //     &self.signature.get_comp_id_safe().unwrap(),
        // );
        // CommandResult::Continue
    }
}

/// Goto
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "stack_stringid", derive(Copy))]
pub struct Goto {
    pub target_state: StringId,
}
impl Goto {
    fn new(args: Vec<String>) -> Result<Self> {
        Ok(Goto {
            target_state: string::new_truncate(&args[0]),
        })
    }
    pub fn execute_loc(&self, comp_state: &mut StringId) -> CommandResult {
        *comp_state = self.target_state.clone();
        CommandResult::Continue
        // CommandResult::GoToState(self.target_state.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "stack_stringid", derive(Copy))]
pub struct Jump {
    pub target: JumpTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "stack_stringid", derive(Copy))]
pub enum JumpTarget {
    Line(u16),
    Tag(StringId),
}

impl Jump {
    fn new(args: Vec<String>) -> Result<Self> {
        if let Ok(num) = args[0].parse::<u16>() {
            Ok(Jump {
                target: JumpTarget::Line(num),
            })
        } else {
            let tag = if args[0].starts_with('@') {
                string::new_truncate(&args[0][1..])
            } else {
                string::new_truncate(&args[0])
            };
            Ok(Jump {
                target: JumpTarget::Tag(tag),
            })
        }
    }
    pub fn execute_loc(&self) -> CommandResult {
        match &self.target {
            JumpTarget::Line(line) => CommandResult::JumpToLine(*line as usize),
            JumpTarget::Tag(tag) => CommandResult::JumpToTag(tag.clone()),
        }
    }
}

/// Invoke
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoke {
    pub events: Vec<StringId>,
}
impl Invoke {
    pub fn new(args: Vec<String>) -> Result<Self> {
        let mut events = Vec::new();
        for arg in &args {
            events.push(string::new_truncate(arg));
        }
        Ok(Invoke { events })
    }
}
impl Invoke {
    pub fn execute_loc(&self) -> CommandResult {
        return CommandResult::ExecCentralExt(CentralRemoteCommand::Invoke(self.clone()));
    }
    pub fn execute_ext(&self, sim: &mut Sim) -> Result<()> {
        for event in &self.events {
            if !sim.event_queue.contains(event) {
                sim.event_queue.push(event.to_owned());
            }
        }
        Ok(())
    }
    pub fn execute_ext_distr(&self, central: &mut SimCentral) -> Result<()> {
        for event in &self.events {
            if !central.event_queue.contains(event) {
                central.event_queue.push(event.to_owned());
            }
        }
        Ok(())
    }
}

/// Spawn
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "stack_stringid", derive(Copy))]
pub struct Spawn {
    pub prefab: Option<StringId>,
    pub spawn_id: Option<StringId>,
    pub out: Option<ShortLocalAddress>,
}
impl Spawn {
    fn new(args: Vec<String>, location: &LocationInfo) -> Result<Self> {
        let matches = getopts::Options::new()
            .optopt("o", "out", "", "")
            .parse(&args)
            .map_err(|e| Error::new(location.clone(), ErrorKind::ParseError(e.to_string())))?;

        let out = matches
            .opt_str("out")
            .map(|s| ShortLocalAddress::from_str(&s))
            .transpose()?;

        if matches.free.len() == 0 {
            Ok(Self {
                prefab: None,
                spawn_id: None,
                out,
            })
        } else if matches.free.len() == 1 {
            Ok(Self {
                prefab: Some(string::new_truncate(&args[0])),
                spawn_id: None,
                out,
            })
        } else if matches.free.len() == 2 {
            Ok(Self {
                prefab: Some(string::new_truncate(&args[0])),
                spawn_id: Some(string::new_truncate(&args[1])),
                out,
            })
        } else {
            return Err(Error::new(
                location.clone(),
                ErrorKind::InvalidCommandBody("can't accept more than 2 arguments".to_string()),
            ));
        }
    }

    pub fn execute_loc(&self) -> CommandResult {
        CommandResult::ExecCentralExt(CentralRemoteCommand::Spawn(self.clone()))
    }

    pub fn execute_ext(&self, sim: &mut Sim, ent_uid: &EntityId) -> Result<()> {
        sim.spawn_entity(self.prefab.as_ref(), self.spawn_id.clone())?;
        // #[cfg(feature = "machine_lua")]
        // sim.setup_lua_state_ent();
        Ok(())
    }
    pub fn execute_ext_distr(&self, central: &mut SimCentral) -> Result<()> {
        central.spawn_entity(
            self.prefab.clone(),
            self.spawn_id.clone(),
            DistributionPolicy::Random,
        )?;
        Ok(())
    }
}
