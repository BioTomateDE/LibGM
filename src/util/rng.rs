pub struct CSharpRng {
    seed_array: [i32; 56],
    inext: usize,
    inextp: usize,
}

impl CSharpRng {
    pub fn new(seed: i32) -> Self {
        let mut num_array: [i32; 56] = [0; 56];
        let mut num1: i32 = 161803398_i32.wrapping_sub(if seed == i32::MIN {i32::MAX} else {seed.abs()});
        num_array[55] = num1;
        let mut num2: i32 = 1;
        let mut index1: usize = 0;

        for _ in 1..55 {
            index1 += 21;
            if index1 >= 55 {
                index1 -= 55;
            }
            num_array[index1] = num2;
            num2 = num1.wrapping_sub(num2);
            if num2 < 0 {
                num2 = num2.wrapping_add(i32::MAX);
            }
            num1 = num_array[index1];
        }

        for _ in 1..5 {
            for index4 in 1..56 {
                let mut num3: usize = index4 + 30;
                if num3 >= 55 {
                    num3 -= 55;
                }
                num_array[index4] = num_array[index4].wrapping_sub(num_array[num3 + 1]);
                if num_array[index4] < 0 {
                    num_array[index4] = num_array[index4].wrapping_add(i32::MAX);
                }
            }
        }

        Self {
            seed_array: num_array,
            inext: 0,
            inextp: 21,
        }
    }

    pub fn next(&mut self) -> i32 {
        let mut index1: usize = self.inext + 1;
        if index1 >= 56 {
            index1 = 1;
        }
        let mut index2: usize = self.inextp + 1;
        if index2 >= 56 {
            index2 = 1;
        }
        let mut num: i32 = self.seed_array[index1].wrapping_sub(self.seed_array[index2]);
        if num == i32::MAX {
            num -= 1;
        }
        if num < 0 {
            num = num.wrapping_add(i32::MAX);
        }
        self.seed_array[index1] = num;
        self.inext = index1;
        self.inextp = index2;
        num
    }
}

