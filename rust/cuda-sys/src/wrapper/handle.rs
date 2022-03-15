use std::ptr::null_mut;

use crate::bindings::cudnnHandle_t;
use crate::bindings::{
    cublasCreate_v2, cublasDestroy_v2, cublasHandle_t, cublasSetStream_v2, cudaEventRecord, cudaGetDeviceCount,
    cudaSetDevice, cudaStreamBeginCapture, cudaStreamCaptureMode, cudaStreamCreate, cudaStreamDestroy,
    cudaStreamEndCapture, cudaStreamSynchronize, cudaStreamWaitEvent, cudaStream_t, cudnnCreate, cudnnDestroy,
    cudnnSetStream,
};
use crate::wrapper::event::CudaEvent;
use crate::wrapper::graph::CudaGraph;
use crate::wrapper::mem::device::DevicePtr;
use crate::wrapper::status::Status;

pub fn cuda_device_count() -> i32 {
    unsafe {
        let mut count = 0;
        cudaGetDeviceCount(&mut count as *mut _).unwrap();
        count
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Device(i32);

impl Device {
    pub fn new(device: i32) -> Self {
        assert!(
            0 <= device && device < cuda_device_count(),
            "Device doesn't exist {}",
            device
        );
        Device(device)
    }

    pub fn all() -> impl Iterator<Item = Self> {
        (0..cuda_device_count()).map(Device::new)
    }

    pub fn inner(self) -> i32 {
        self.0
    }

    // Set the current cuda device to this device.
    //TODO is this enough when there are multiple threads running?
    pub unsafe fn switch_to(self) {
        cudaSetDevice(self.0).unwrap()
    }

    pub fn alloc(self, len_bytes: usize) -> DevicePtr {
        DevicePtr::alloc(self, len_bytes)
    }
}

//TODO copy? clone? default stream?
#[derive(Debug)]
pub struct CudaStream {
    device: Device,
    inner: cudaStream_t,
}

impl Drop for CudaStream {
    fn drop(&mut self) {
        unsafe {
            cudaStreamDestroy(self.inner).unwrap_in_drop();
        }
    }
}

impl CudaStream {
    pub fn new(device: Device) -> Self {
        unsafe {
            let mut inner = null_mut();
            device.switch_to();
            cudaStreamCreate(&mut inner as *mut _).unwrap();
            CudaStream { device, inner }
        }
    }

    pub unsafe fn synchronize(&self) {
        cudaStreamSynchronize(self.inner()).unwrap()
    }

    pub fn device(&self) -> Device {
        self.device
    }

    pub unsafe fn inner(&self) -> cudaStream_t {
        self.inner
    }

    pub unsafe fn record_event(&self, event: &CudaEvent) {
        cudaEventRecord(event.inner(), self.inner()).unwrap()
    }

    pub unsafe fn record_new_event(&self) -> CudaEvent {
        let event = CudaEvent::new();
        self.record_event(&event);
        event
    }

    pub unsafe fn wait_for_event(&self, event: &CudaEvent) {
        cudaStreamWaitEvent(self.inner, event.inner(), 0).unwrap();
    }

    pub unsafe fn begin_capture(&self) {
        cudaStreamBeginCapture(self.inner(), cudaStreamCaptureMode::cudaStreamCaptureModeGlobal).unwrap()
    }

    pub unsafe fn end_capture(&self) -> CudaGraph {
        let mut graph = null_mut();
        cudaStreamEndCapture(self.inner(), &mut graph as *mut _).unwrap();
        CudaGraph::new_from_inner(graph)
    }
}

#[derive(Debug)]
pub struct CudnnHandle {
    inner: cudnnHandle_t,
    stream: CudaStream,
}

impl Drop for CudnnHandle {
    fn drop(&mut self) {
        unsafe {
            self.device().switch_to();
            cudnnDestroy(self.inner).unwrap_in_drop()
        }
    }
}

impl CudnnHandle {
    pub fn new(device: Device) -> Self {
        CudnnHandle::new_with_stream(CudaStream::new(device))
    }

    pub fn new_with_stream(stream: CudaStream) -> Self {
        unsafe {
            let mut inner = null_mut();
            stream.device.switch_to();
            cudnnCreate(&mut inner as *mut _).unwrap();
            cudnnSetStream(inner, stream.inner()).unwrap();
            CudnnHandle { inner, stream }
        }
    }

    pub fn device(&self) -> Device {
        self.stream.device()
    }

    pub unsafe fn stream(&self) -> &CudaStream {
        &self.stream
    }

    pub unsafe fn inner(&self) -> cudnnHandle_t {
        self.inner
    }
}

#[derive(Debug)]
pub struct CublasHandle {
    inner: cublasHandle_t,
    stream: CudaStream,
}

impl Drop for CublasHandle {
    fn drop(&mut self) {
        unsafe { cublasDestroy_v2(self.inner).unwrap_in_drop() }
    }
}

impl CublasHandle {
    pub fn new(device: Device) -> Self {
        CublasHandle::new_with_stream(CudaStream::new(device))
    }

    pub fn new_with_stream(stream: CudaStream) -> Self {
        unsafe {
            let mut inner = null_mut();
            stream.device.switch_to();
            cublasCreate_v2(&mut inner as *mut _).unwrap();
            cublasSetStream_v2(inner, stream.inner()).unwrap();
            CublasHandle { inner, stream }
        }
    }

    pub unsafe fn stream(&self) -> &CudaStream {
        &self.stream
    }

    pub unsafe fn inner(&self) -> cublasHandle_t {
        self.inner
    }
}
