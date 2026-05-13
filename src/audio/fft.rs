
use rodio::Source;
use rustfft::num_traits::ToPrimitive;
use std::sync::{Arc, Mutex};
use std::num::{NonZeroU16, NonZeroU32}; 

pub type SharedSamples = Arc<Mutex<(Vec<f32>, usize)>>;

pub struct FftInterceptor<S> {
    source: S,
    shared_samples: SharedSamples,
}

impl<S> FftInterceptor<S> {
    pub fn new(source: S, shared_samples: SharedSamples) -> Self {
        Self { source, shared_samples }
    }
}


impl<S: Source> Iterator for FftInterceptor<S> 
where 
    S::Item: ToPrimitive, 
{
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.source.next()?;

        if let Ok(mut guard) = self.shared_samples.lock() {
            let cursor = guard.1; 
            guard.0[cursor] = sample.to_f32().unwrap_or(0.0);
            guard.1 = (cursor + 1) % guard.0.len();
        }

        Some(sample)
    }
}

impl<S: Source> Source for FftInterceptor<S> 
where 
    S::Item: ToPrimitive, 
{
    fn current_span_len(&self) -> Option<usize> {
        self.source.current_span_len()
    }

    fn channels(&self) -> NonZeroU16 { 
        self.source.channels() 
    }
    
    fn sample_rate(&self) -> NonZeroU32 { 
        self.source.sample_rate() 
    }
    
    fn total_duration(&self) -> Option<std::time::Duration> { 
        self.source.total_duration() 
    }
}

