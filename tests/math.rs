use std::ops::Add;

use custos::{libs::{cpu::CPU, opencl::{CLDevice, api::OCLError}}, Buffer, AsDev, Matrix};


/* 
#[test]
fn add() -> Result<(), OCLError> {
    
    let device = CPU.select();
    
    let lhs = Buffer::from((&device, [4., 1., 2.,]));
    let rhs = Buffer::from((&device, [4., 1., 2.,]));

    let native = lhs + rhs;



    let device = CLDevice::get(0)?.select();
    
    let lhs = Buffer::from((&device, [4., 1., 2.,]));
    let rhs = Buffer::from((&device, [4., 1., 2.,]));

    let opencl = lhs + rhs;
    
    assert_eq!(opencl, native);
    Ok(())   
}
*/

#[test]
fn add() {
    CLDevice::get(0).unwrap().select();

    let a = Matrix::from(( (1, 3), &[1., 4., 2.,] ));
    let b = Matrix::from(( (1, 3), &[1., 4., 2.,] ));

    let c = a + b;

}