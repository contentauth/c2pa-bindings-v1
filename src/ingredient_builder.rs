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

use std::sync::RwLock;

use c2pa::Ingredient;

use crate::{
    error::{C2paError, Result},
    Stream, StreamAdapter,
};

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

    // pub fn from_json(&self, json: &str) -> Result<()> {
    //     let ingredient = Ingredient::from_json(json).map_err(C2paError::Sdk)?;
    //     if let Ok(mut m) = self.ingredient.try_write() {
    //         *m = ingredient;
    //     } else {
    //         return Err(C2paError::RwLock);
    //     };
    //     Ok(())
    // }

    pub fn read(&self, format: &str, stream: &mut dyn c2pa::CAIRead) -> Result<String> {
        let ingredient = Ingredient::from_stream(format, stream).map_err(C2paError::Sdk)?;
        if let Ok(mut i) = self.ingredient.try_write() {
            let json = ingredient.to_string();
            *i = ingredient;
            Ok(json)
        } else {
            return Err(C2paError::RwLock);
        }
    }

    pub fn json(&self) -> Result<String> {
        if let Ok(i) = self.ingredient.try_read() {
            Ok(i.to_string())
        } else {
            return Err(C2paError::RwLock);
        }
    }

    pub fn resource(&self, id: &str) -> Result<Vec<u8>> {
        if let Ok(ingredient) = self.ingredient.try_read() {
            match ingredient.resources().get(id) {
                Ok(r) => Ok(r.into_owned()),
                Err(e) => Err(C2paError::Sdk(e)),
            }
        } else {
            Err(C2paError::RwLock)
        }
    }

    pub fn resource_write_stream(&self, id: &str, stream: &dyn Stream) -> Result<()> {
        let mut stream = StreamAdapter::from(stream);
        self.resource_write(id, &mut stream)
    }

    pub fn resource_write(&self, id: &str, stream: &mut dyn c2pa::CAIReadWrite) -> Result<()> {
        if let Ok(ingredient) = self.ingredient.try_read() {
            match ingredient.resources().get(id) {
                Ok(bytes) => {
                    stream.write_all(&bytes).map_err(C2paError::Io)?;
                    Ok(())
                }
                Err(e) => Err(C2paError::Sdk(e)),
            }
        } else {
            Err(C2paError::RwLock)
        }
    }
}
