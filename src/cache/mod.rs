use std::clone::Clone;
use std::fmt::Debug;
use std::hash::Hash;
use anyhow::Result;
use mini_moka::unsync::Cache;

pub trait CacheExt<K, V, CB: Fn() -> Result<V>> {
    fn get_or_create(&mut self, key: &K, cb: CB) -> Result<V>;
}

impl<K, V, CB: Fn() -> Result<V>> CacheExt<K, V, CB> for Cache<K,V> 
where
    K: Clone + Debug + Eq + Hash,
    V: Clone,
{
    fn get_or_create(&mut self, key: &K, cb: CB) -> Result<V> 
    {
        match self.get(key) {
            Some(v) => Ok(v.clone()),
            None => {
                match cb() {
                    Ok(v) => {
                        let res = v.clone();
                        self.insert(key.clone(), v);
                        Ok(res)
                    },
                    Err(e) => Err(e)
                }
            }
        }
    }
}