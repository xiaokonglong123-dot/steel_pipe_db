use crate::domain::value_objects::{PipeNumber, HeatNumber};
use crate::domain::events::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Seamless pipe entity - core domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeamlessPipe {
    id: Uuid,
    pipe_number: PipeNumber,
    heat_number: HeatNumber,
    outer_diameter: f64,
    wall_thickness: f64,
    length: f64,
    weight: f64,
    grade: Api5ctGrade,
    steel_grade: String,
    heat_treatment: HeatTreatment,
    thread_type: ThreadType,
    end_type: EndType,
    status: PipeStatus,
    location_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Api5ctGrade {
    J55,
    K55,
    N80,
    L80,
    C90,
    T95,
    P110,
    Q125,
}

impl fmt::Display for Api5ctGrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeatTreatment {
    Normalized,
    NormalizedTempered,
    QuenchedTempered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadType {
    BTC,
    LC,
    SC,
    NC,
    VAM,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndType {
    Plain,
    Threaded,
    Coupled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipeStatus {
    New,
    InStock,
    Outbound,
    Scrapped,
}

impl fmt::Display for PipeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self).map(|_| {
            // Convert to lowercase for DB storage
            match self {
                PipeStatus::New => "new",
                PipeStatus::InStock => "in_stock",
                PipeStatus::Outbound => "outbound",
                PipeStatus::Scrapped => "scrapped",
            }
        })?;
        Ok(())
    }
}

impl SeamlessPipe {
    pub fn new(cmd: CreateSeamlessPipeCommand) -> Result<(Self, Vec<DomainEvent>), crate::error::AppError> {
        let weight = Self::calculate_weight(cmd.outer_diameter, cmd.wall_thickness, cmd.length);
        
        let pipe = Self {
            id: Uuid::new_v4(),
            pipe_number: cmd.pipe_number,
            heat_number: cmd.heat_number,
            outer_diameter: cmd.outer_diameter,
            wall_thickness: cmd.wall_thickness,
            length: cmd.length,
            weight,
            grade: cmd.grade,
            steel_grade: cmd.steel_grade,
            heat_treatment: cmd.heat_treatment,
            thread_type: cmd.thread_type,
            end_type: cmd.end_type,
            status: PipeStatus::New,
            location_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };
        
        let event = DomainEvent::PipeCreated { 
            pipe_id: pipe.id, 
            pipe_number: pipe.pipe_number.clone() 
        };
        Ok((pipe, vec![event]))
    }

    pub fn move_to_location(&mut self, location_id: Uuid) -> Vec<DomainEvent> {
        self.location_id = Some(location_id);
        self.status = PipeStatus::InStock;
        self.updated_at = Utc::now();
        vec![DomainEvent::PipeMoved { 
            pipe_id: self.id, 
            location_id 
        }]
    }

    pub fn mark_outbound(&mut self) -> Vec<DomainEvent> {
        self.status = PipeStatus::Outbound;
        self.updated_at = Utc::now();
        vec![DomainEvent::PipeOutbound { 
            pipe_id: self.id 
        }]
    }

    pub fn mark_scrapped(&mut self) -> Vec<DomainEvent> {
        self.status = PipeStatus::Scrapped;
        self.updated_at = Utc::now();
        vec![DomainEvent::PipeScrapped { 
            pipe_id: self.id 
        }]
    }

    fn calculate_weight(od: f64, wt: f64, len: f64) -> f64 {
        // API 5CT formula: (OD - WT) * WT * 0.02466 * length
        (od - wt) * wt * 0.02466 * len
    }

    // Getters
    pub fn id(&self) -> Uuid { self.id }
    pub fn pipe_number(&self) -> &PipeNumber { &self.pipe_number }
    pub fn heat_number(&self) -> &HeatNumber { &self.heat_number }
    pub fn outer_diameter(&self) -> f64 { self.outer_diameter }
    pub fn wall_thickness(&self) -> f64 { self.wall_thickness }
    pub fn length(&self) -> f64 { self.length }
    pub fn weight(&self) -> f64 { self.weight }
    pub fn grade(&self) -> Api5ctGrade { self.grade }
    pub fn steel_grade(&self) -> &str { &self.steel_grade }
    pub fn heat_treatment(&self) -> HeatTreatment { self.heat_treatment }
    pub fn thread_type(&self) -> ThreadType { self.thread_type }
    pub fn end_type(&self) -> EndType { self.end_type }
    pub fn status(&self) -> PipeStatus { self.status }
    pub fn location_id(&self) -> Option<Uuid> { self.location_id }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    pub fn deleted_at(&self) -> Option<DateTime<Utc>> { self.deleted_at }
    
    pub fn set_deleted_at(&mut self, deleted_at: DateTime<Utc>) {
        self.deleted_at = Some(deleted_at);
        self.updated_at = deleted_at;
    }
}

#[derive(Debug, Clone)]
pub struct CreateSeamlessPipeCommand {
    pub pipe_number: PipeNumber,
    pub heat_number: HeatNumber,
    pub outer_diameter: f64,
    pub wall_thickness: f64,
    pub length: f64,
    pub grade: Api5ctGrade,
    pub steel_grade: String,
    pub heat_treatment: HeatTreatment,
    pub thread_type: ThreadType,
    pub end_type: EndType,
}

use std::fmt;