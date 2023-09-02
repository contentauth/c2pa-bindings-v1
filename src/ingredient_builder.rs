// Copyright 2023 Adobe. All rights reserved.
// This file is licensed to you under the Apache License,
// Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
// or the MIT license (http://opensource.org/licenses/MIT),
// at your option.

// Unless required by applicable law or agreed to in writing,
// this software is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR REPRESENTATIONS OF ANY KIND, either express or
// implied. See the LICENSE-MIT and LICENSE-APACHE files for the
// specific language governing permissions and limitations under
// each license.

use std::io::{Read, Seek, Write};
use std::sync::RwLock;

use crate::error::Result;

use c2pa::Ingredient;

pub struct IngredientBuilderSettings {}

pub struct IngredientBuilder {
    pub settings: IngredientBuilderSettings,
    pub ingredient: RwLock<Ingredient>,
}

impl IngredientBuilder {
    pub fn new(settings: IngredientBuilderSettings) -> Self {
        Self {
            settings,
            ingredient: RwLock::new(Ingredient::new("test", "image/jpeg", "instance_id")),
        }
    }
    pub fn from_stream(self, _input: impl Read + Seek) -> Self {
        self
    }

    pub fn c2pa_data(_stream: Option<impl Write>) -> Result<()> {
        Ok(())
    }
}
