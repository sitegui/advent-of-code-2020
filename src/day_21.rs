use crate::data::Data;
use crate::parser::Parser;
use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet};
use std::mem;

#[derive(Debug, Clone)]
struct Ingredient<'a> {
    name: &'a [u8],
    allergen: Option<&'a [u8]>,
    possible_allergens: BTreeSet<&'a [u8]>,
}

#[derive(Debug, Clone)]
struct Allergen<'a> {
    name: &'a [u8],
    ingredient: Option<&'a [u8]>,
    possible_ingredients: BTreeSet<&'a [u8]>,
}

#[derive(Debug)]
struct Food<'a> {
    ingredients: BTreeSet<&'a [u8]>,
    allergens: BTreeSet<&'a [u8]>,
}

#[derive(Debug, Default)]
struct Db<'a> {
    ingredients: BTreeMap<&'a [u8], Ingredient<'a>>,
    allergens: BTreeMap<&'a [u8], Allergen<'a>>,
}

pub fn solve() -> (i64, i64) {
    let data = Data::read(21);

    let mut db = Db::default();

    let foods = data
        .lines()
        .map(|line| Food::parse(line, &mut db))
        .collect_vec();

    db.mark_possible_relations(&foods);

    let safe_ingredients = db
        .ingredients
        .values()
        .filter(|ingredient| ingredient.possible_allergens.is_empty())
        .map(|ingredient| ingredient.name)
        .collect_vec();

    let mut part_1 = 0;
    for ingredient in safe_ingredients {
        for food in &foods {
            if food.ingredients.contains(&ingredient) {
                part_1 += 1;
            }
        }
    }

    db.solve_allergies();

    let danger_list = db
        .allergens
        .values()
        .map(|allergen| std::str::from_utf8(allergen.ingredient.unwrap()).unwrap())
        .join(",");

    if cfg!(debug_assertions) {
        // Only print on debug mode
        println!("danger_list = {:?}", danger_list);
    }
    (part_1, danger_list.len() as i64)
}

impl<'a> Food<'a> {
    fn parse(mut line: &'a [u8], db: &mut Db<'a>) -> Self {
        let ingredients = line
            .consume_until(b'(')
            .split_byte(b' ', true)
            .inspect(|name| db.ensure_ingredient(name))
            .collect();

        line.consume_prefix(b"contains ");
        let allergens = line
            .consume_until(b')')
            .split_bytes(b", ", true)
            .inspect(|name| db.ensure_allergen(name))
            .collect();

        Food {
            ingredients,
            allergens,
        }
    }
}

impl<'a> Db<'a> {
    fn ensure_ingredient(&mut self, name: &'a [u8]) {
        self.ingredients.entry(name).or_insert_with(|| Ingredient {
            name,
            allergen: None,
            possible_allergens: Default::default(),
        });
    }

    fn ensure_allergen(&mut self, name: &'a [u8]) {
        self.allergens.entry(name).or_insert_with(|| Allergen {
            name,
            ingredient: None,
            possible_ingredients: Default::default(),
        });
    }

    fn mark_possible_relations(&mut self, foods: &[Food<'a>]) {
        for allergen in self.allergens.values_mut() {
            let mut possible_ingredients: BTreeSet<_> = self.ingredients.keys().copied().collect();
            for food in foods {
                if food.allergens.contains(allergen.name) {
                    // This food contains this allergen. Since each allergen can only come from a single
                    // source, we not its source is in this list.
                    possible_ingredients = possible_ingredients
                        .intersection(&food.ingredients)
                        .copied()
                        .collect();
                }
            }

            for &ingredient in &possible_ingredients {
                self.ingredients
                    .get_mut(ingredient)
                    .unwrap()
                    .possible_allergens
                    .insert(allergen.name);
            }

            allergen.possible_ingredients = possible_ingredients;
        }
    }

    fn solve_allergies(&mut self) {
        let mut pending_allergens = self.allergens.keys().copied().collect_vec();
        let mut pending_ingredients = self.ingredients.keys().copied().collect_vec();

        while !pending_allergens.is_empty() || !pending_ingredients.is_empty() {
            pending_allergens = pending_allergens
                .iter()
                .copied()
                .filter(|&allergen_name| {
                    let allergen = self.allergens.get_mut(allergen_name).unwrap();
                    if allergen.ingredient.is_some() {
                        return false;
                    }
                    match allergen.possible_ingredients.len() {
                        0 => false,
                        1 => {
                            let ingredient = *allergen.possible_ingredients.iter().next().unwrap();
                            self.mark_relation(
                                allergen_name,
                                ingredient,
                                &pending_allergens,
                                &pending_ingredients,
                            );
                            false
                        }
                        _ => true,
                    }
                })
                .collect_vec();

            pending_ingredients = pending_ingredients
                .iter()
                .copied()
                .filter(|&ingredient_name| {
                    let ingredient = self.ingredients.get_mut(ingredient_name).unwrap();
                    if ingredient.allergen.is_some() {
                        return false;
                    }
                    match ingredient.possible_allergens.len() {
                        0 => false,
                        1 => {
                            let allergen = *ingredient.possible_allergens.iter().next().unwrap();
                            self.mark_relation(
                                allergen,
                                ingredient_name,
                                &pending_allergens,
                                &pending_ingredients,
                            );
                            false
                        }
                        _ => true,
                    }
                })
                .collect_vec();
        }
    }

    fn mark_relation(
        &mut self,
        allergen: &'a [u8],
        ingredient: &'a [u8],
        pending_allergens: &[&'a [u8]],
        pending_ingredients: &[&'a [u8]],
    ) {
        for &allergen in pending_allergens {
            self.allergens
                .get_mut(allergen)
                .unwrap()
                .possible_ingredients
                .remove(ingredient);
        }
        for &ingredient in pending_ingredients {
            self.ingredients
                .get_mut(ingredient)
                .unwrap()
                .possible_allergens
                .remove(allergen);
        }

        assert!(mem::replace(
            &mut self.allergens.get_mut(allergen).unwrap().ingredient,
            Some(ingredient)
        )
        .is_none());
        assert!(mem::replace(
            &mut self.ingredients.get_mut(ingredient).unwrap().allergen,
            Some(allergen)
        )
        .is_none());
    }
}
