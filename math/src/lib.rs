pub use impls::{Rotor, Transform};

mod impls {
    use bytemuck::{Pod, Zeroable};

    ga_generator::ga! {
        element_type = f32;
        scalar_name = s;
        elements = [e0 = zero, e1 = positive_one, e2 = positive_one, e3 = positive_one, e4 = positive_one];

        group Scalar = s;

        group VgaVector      = e1 + e2 + e3 + e4;
        group VgaBivector    = VgaVector ^ VgaVector;
        group VgaTrivector   = VgaVector ^ VgaBivector;
        group VgaQuadvector  = VgaVector ^ VgaTrivector;
        group VgaPentavector = VgaVector ^ VgaQuadvector;

        group #[derive(Zeroable, Pod)] #[repr(C)] Rotor = Scalar + VgaBivector + VgaQuadvector;

        group RotorSquaredMagnitude = Scalar + VgaQuadvector;
        fn rotor_squared_magnitude(rotor: Rotor) -> RotorSquaredMagnitude {
            return ~rotor * rotor;
        }

        fn rotor_then(a: Rotor, b: Rotor) -> Rotor {
            return a * b;
        }

        fn rotor_reverse(rotor: Rotor) -> Rotor {
            return ~rotor;
        }

        fn rotate_direction(rotor: Rotor, x: Scalar, y: Scalar, z: Scalar, w: Scalar) -> [Scalar, Scalar, Scalar, Scalar] {
            let x = e1 - x*e0;
            let y = e2 - y*e0;
            let z = e3 - z*e0;
            let w = e4 - w*e0;
            let origin = ((e1 ^ e2) ^ e3) ^ e4;
            // join the point to the origin to make a line, then get the lines intersection with the hyperplane at infinity
            let point = (origin & (((x ^ y) ^ z) ^ w)) ^ e0;

            let transformed = (~rotor * point) * rotor;

            // without this it tries to return an extra scalar
            let assume_normalised_rotor = point | (1 - (~rotor * rotor));

            let result = transformed + assume_normalised_rotor;

            return [
                result & e1,
                result & e2,
                result & e3,
                result & e4,
            ];
        }

        fn rotor_x(rotor: Rotor) -> [Scalar, Scalar, Scalar, Scalar] {
            let x = e1 - 1*e0;
            let y = e2 - 0*e0;
            let z = e3 - 0*e0;
            let w = e4 - 0*e0;
            let origin = ((e1 ^ e2) ^ e3) ^ e4;
            // join the point to the origin to make a line, then get the lines intersection with the hyperplane at infinity
            let point = (origin & (((x ^ y) ^ z) ^ w)) ^ e0;

            let transformed = (~rotor * point) * rotor;

            // without this it tries to return an extra scalar
            let assume_normalised_rotor = point | (1 - (~rotor * rotor));

            let result = transformed + assume_normalised_rotor;

            return [
                result & e1,
                result & e2,
                result & e3,
                result & e4,
            ];
        }

        fn rotor_y(rotor: Rotor) -> [Scalar, Scalar, Scalar, Scalar] {
            let x = e1 - 0*e0;
            let y = e2 - 1*e0;
            let z = e3 - 0*e0;
            let w = e4 - 0*e0;
            let origin = ((e1 ^ e2) ^ e3) ^ e4;
            // join the point to the origin to make a line, then get the lines intersection with the hyperplane at infinity
            let point = (origin & (((x ^ y) ^ z) ^ w)) ^ e0;

            let transformed = (~rotor * point) * rotor;

            // without this it tries to return an extra scalar
            let assume_normalised_rotor = point | (1 - (~rotor * rotor));

            let result = transformed + assume_normalised_rotor;

            return [
                result & e1,
                result & e2,
                result & e3,
                result & e4,
            ];
        }

        fn rotor_z(rotor: Rotor) -> [Scalar, Scalar, Scalar, Scalar] {
            let x = e1 - 0*e0;
            let y = e2 - 0*e0;
            let z = e3 - 1*e0;
            let w = e4 - 0*e0;
            let origin = ((e1 ^ e2) ^ e3) ^ e4;
            // join the point to the origin to make a line, then get the lines intersection with the hyperplane at infinity
            let point = (origin & (((x ^ y) ^ z) ^ w)) ^ e0;

            let transformed = (~rotor * point) * rotor;

            // without this it tries to return an extra scalar
            let assume_normalised_rotor = point | (1 - (~rotor * rotor));

            let result = transformed + assume_normalised_rotor;

            return [
                result & e1,
                result & e2,
                result & e3,
                result & e4,
            ];
        }

        fn rotor_w(rotor: Rotor) -> [Scalar, Scalar, Scalar, Scalar] {
            let x = e1 - 0*e0;
            let y = e2 - 0*e0;
            let z = e3 - 0*e0;
            let w = e4 - 1*e0;
            let origin = ((e1 ^ e2) ^ e3) ^ e4;
            // join the point to the origin to make a line, then get the lines intersection with the hyperplane at infinity
            let point = (origin & (((x ^ y) ^ z) ^ w)) ^ e0;

            let transformed = (~rotor * point) * rotor;

            // without this it tries to return an extra scalar
            let assume_normalised_rotor = point | (1 - (~rotor * rotor));

            let result = transformed + assume_normalised_rotor;

            return [
                result & e1,
                result & e2,
                result & e3,
                result & e4,
            ];
        }

        group PgaVector      = e0 + e1 + e2 + e3 + e4;
        group PgaBivector    = PgaVector ^ PgaVector;
        group PgaTrivector   = PgaVector ^ PgaBivector;
        group PgaQuadvector  = PgaVector ^ PgaTrivector;
        group PgaPentavector = PgaVector ^ PgaQuadvector;

        group #[derive(Zeroable, Pod)] #[repr(C)] Transform = Scalar + PgaBivector + PgaQuadvector;

        group TransformSquaredMagnitude = Scalar + PgaQuadvector;
        fn transform_squared_magnitude(transform: Transform) -> TransformSquaredMagnitude {
            return ~transform * transform;
        }

        fn transform_then(a: Transform, b: Transform) -> Transform {
            return a * b;
        }

        fn transform_reverse(transform: Transform) -> Transform {
            return ~transform;
        }

        fn transform_point(transform: Transform, x: Scalar, y: Scalar, z: Scalar, w: Scalar) -> [Scalar, Scalar, Scalar, Scalar] {
            let x = e1 - x*e0;
            let y = e2 - y*e0;
            let z = e3 - z*e0;
            let w = e4 - w*e0;
            let point = ((x ^ y) ^ z) ^ w;

            let transformed = (~transform * point) * transform;

            // without this it tries to return an extra scalar
            let assume_normalised_transform = point | (1 - (~transform * transform));

            let result = transformed + assume_normalised_transform;

            return [
                result & e1,
                result & e2,
                result & e3,
                result & e4,
            ];
        }

        fn transform_position(transform: Transform) -> [Scalar, Scalar, Scalar, Scalar] {
            let x = e1 - 0*e0;
            let y = e2 - 0*e0;
            let z = e3 - 0*e0;
            let w = e4 - 0*e0;
            let point = ((x ^ y) ^ z) ^ w;

            let transformed = (~transform * point) * transform;

            // without this it tries to return an extra scalar
            let assume_normalised_transform = point | (1 - (~transform * transform));

            let result = transformed + assume_normalised_transform;

            return [
                result & e1,
                result & e2,
                result & e3,
                result & e4,
            ];
        }
    }

    impl Rotor {
        #[inline]
        pub fn identity() -> Self {
            Self {
                s: 1.0,
                ..Self::zero()
            }
        }

        #[inline]
        pub fn rotate_xy(angle: f32) -> Self {
            let (sin, cos) = (angle * 0.5).sin_cos();
            Self {
                s: cos,
                e1e2: sin,
                ..Self::zero()
            }
        }

        #[inline]
        pub fn rotate_xz(angle: f32) -> Self {
            let (sin, cos) = (angle * 0.5).sin_cos();
            Self {
                s: cos,
                e1e3: sin,
                ..Self::zero()
            }
        }

        #[inline]
        pub fn rotate_xw(angle: f32) -> Self {
            let (sin, cos) = (angle * 0.5).sin_cos();
            Self {
                s: cos,
                e1e4: sin,
                ..Self::zero()
            }
        }

        #[inline]
        pub fn rotate_yz(angle: f32) -> Self {
            let (sin, cos) = (angle * 0.5).sin_cos();
            Self {
                s: cos,
                e2e3: sin,
                ..Self::zero()
            }
        }

        #[inline]
        pub fn rotate_yw(angle: f32) -> Self {
            let (sin, cos) = (angle * 0.5).sin_cos();
            Self {
                s: cos,
                e2e4: sin,
                ..Self::zero()
            }
        }

        #[inline]
        pub fn rotate_zw(angle: f32) -> Self {
            let (sin, cos) = (angle * 0.5).sin_cos();
            Self {
                s: cos,
                e3e4: sin,
                ..Self::zero()
            }
        }

        #[inline]
        pub fn then(self, then: Self) -> Self {
            rotor_then(self, then)
        }

        #[inline]
        pub fn reverse(self) -> Self {
            rotor_reverse(self)
        }

        #[inline]
        pub fn transform_direction(self, direction: cgmath::Vector4<f32>) -> cgmath::Vector4<f32> {
            let (Scalar { s: x }, Scalar { s: y }, Scalar { s: z }, Scalar { s: w }) =
                rotate_direction(
                    self,
                    Scalar { s: direction.x },
                    Scalar { s: direction.y },
                    Scalar { s: direction.z },
                    Scalar { s: direction.w },
                );
            cgmath::Vector4 { x, y, z, w }
        }

        #[inline]
        pub fn forward(self) -> cgmath::Vector4<f32> {
            let (Scalar { s: x }, Scalar { s: y }, Scalar { s: z }, Scalar { s: w }) =
                rotor_x(self);
            cgmath::Vector4 { x, y, z, w }
        }

        #[inline]
        pub fn up(self) -> cgmath::Vector4<f32> {
            let (Scalar { s: x }, Scalar { s: y }, Scalar { s: z }, Scalar { s: w }) =
                rotor_y(self);
            cgmath::Vector4 { x, y, z, w }
        }

        #[inline]
        pub fn right(self) -> cgmath::Vector4<f32> {
            let (Scalar { s: x }, Scalar { s: y }, Scalar { s: z }, Scalar { s: w }) =
                rotor_z(self);
            cgmath::Vector4 { x, y, z, w }
        }

        #[inline]
        pub fn ana(self) -> cgmath::Vector4<f32> {
            let (Scalar { s: x }, Scalar { s: y }, Scalar { s: z }, Scalar { s: w }) =
                rotor_w(self);
            cgmath::Vector4 { x, y, z, w }
        }
    }

    impl Transform {
        #[inline]
        pub fn identity() -> Self {
            Self {
                s: 1.0,
                ..Self::zero()
            }
        }

        #[inline]
        pub fn translation(offset: cgmath::Vector4<f32>) -> Self {
            Self {
                s: 1.0,
                e0e1: offset.x * 0.5,
                e0e2: offset.y * 0.5,
                e0e3: offset.z * 0.5,
                e0e4: offset.w * 0.5,
                ..Self::zero()
            }
        }

        #[inline]
        pub fn rotate_xy(angle: f32) -> Self {
            Self::from_rotor(Rotor::rotate_xy(angle))
        }

        #[inline]
        pub fn rotate_xz(angle: f32) -> Self {
            Self::from_rotor(Rotor::rotate_xz(angle))
        }

        #[inline]
        pub fn rotate_xw(angle: f32) -> Self {
            Self::from_rotor(Rotor::rotate_xw(angle))
        }

        #[inline]
        pub fn rotate_yz(angle: f32) -> Self {
            Self::from_rotor(Rotor::rotate_yz(angle))
        }

        #[inline]
        pub fn rotate_yw(angle: f32) -> Self {
            Self::from_rotor(Rotor::rotate_yw(angle))
        }

        #[inline]
        pub fn rotate_zw(angle: f32) -> Self {
            Self::from_rotor(Rotor::rotate_zw(angle))
        }

        #[inline]
        pub fn then(self, then: Self) -> Self {
            transform_then(self, then)
        }

        #[inline]
        pub fn reverse(self) -> Self {
            transform_reverse(self)
        }

        #[inline]
        pub fn transform_point(self, point: cgmath::Vector4<f32>) -> cgmath::Vector4<f32> {
            let (Scalar { s: x }, Scalar { s: y }, Scalar { s: z }, Scalar { s: w }) =
                transform_point(
                    self,
                    Scalar { s: point.x },
                    Scalar { s: point.y },
                    Scalar { s: point.z },
                    Scalar { s: point.w },
                );
            cgmath::Vector4 { x, y, z, w }
        }

        #[inline]
        pub fn transform_direction(self, direction: cgmath::Vector4<f32>) -> cgmath::Vector4<f32> {
            self.rotor_part().transform_direction(direction)
        }

        #[inline]
        pub fn position(self) -> cgmath::Vector4<f32> {
            let (Scalar { s: x }, Scalar { s: y }, Scalar { s: z }, Scalar { s: w }) =
                transform_position(self);
            cgmath::Vector4 { x, y, z, w }
        }

        #[inline]
        pub fn forward(self) -> cgmath::Vector4<f32> {
            self.rotor_part().forward()
        }

        #[inline]
        pub fn up(self) -> cgmath::Vector4<f32> {
            self.rotor_part().up()
        }

        #[inline]
        pub fn right(self) -> cgmath::Vector4<f32> {
            self.rotor_part().right()
        }

        #[inline]
        pub fn ana(self) -> cgmath::Vector4<f32> {
            self.rotor_part().ana()
        }

        #[inline]
        pub fn from_rotor(rotor: Rotor) -> Self {
            let Rotor {
                s,
                e1e2,
                e1e3,
                e1e4,
                e2e3,
                e2e4,
                e3e4,
                e1e2e3e4,
            } = rotor;
            Self {
                s,
                e0e1: 0.0,
                e0e2: 0.0,
                e0e3: 0.0,
                e0e4: 0.0,
                e1e2,
                e1e3,
                e1e4,
                e2e3,
                e2e4,
                e3e4,
                e0e1e2e3: 0.0,
                e0e1e2e4: 0.0,
                e0e1e3e4: 0.0,
                e0e2e3e4: 0.0,
                e1e2e3e4,
            }
        }

        #[inline]
        pub fn rotor_part(self) -> Rotor {
            let Self {
                s,
                e0e1: _,
                e0e2: _,
                e0e3: _,
                e0e4: _,
                e1e2,
                e1e3,
                e1e4,
                e2e3,
                e2e4,
                e3e4,
                e0e1e2e3: _,
                e0e1e2e4: _,
                e0e1e3e4: _,
                e0e2e3e4: _,
                e1e2e3e4,
            } = self;
            Rotor {
                s,
                e1e2,
                e1e3,
                e1e4,
                e2e3,
                e2e4,
                e3e4,
                e1e2e3e4,
            }
        }
    }
}
