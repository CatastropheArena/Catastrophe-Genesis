use rand::Rng;

/// 打乱数组中元素的顺序
///
/// # 参数
///
/// * `array` - 需要打乱的数组
///
/// # 返回值
///
/// 返回打乱后的新数组
pub fn shuffle<T: Clone>(array: &[T]) -> Vec<T> {
    let mut rng = rand::thread_rng();
    let mut shuffled = array.to_vec();

    for i in (1..array.len()).rev() {
        let j = rng.gen_range(0..=i);
        shuffled.swap(i, j);
    }

    shuffled
}

/// 将数组分割成指定大小的块
///
/// # 参数
///
/// * `array` - 要分割的数组
/// * `chunk_size` - 每个块的大小
///
/// # 返回值
///
/// 返回分割后的二维数组
pub fn split_into_chunks<T: Clone>(array: &[T], chunk_size: usize) -> Vec<Vec<T>> {
    (0..array.len())
        .step_by(chunk_size)
        .map(|begin| {
            let end = (begin + chunk_size).min(array.len());
            array[begin..end].to_vec()
        })
        .collect()
}

/// ELO评分系统模块，提供评分计算功能
pub mod elo {
    /// 定义比赛结果的枚举
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum MatchOutcome {
        Victory,
        Defeat,
        // 可扩展添加其他结果，如Draw
    }

    /// ELO系统配置的特性
    pub trait EloConfig {
        /// 获取性能常数
        fn performance_constant(&self) -> f64;
        
        /// 获取K因子
        fn k_factor(&self) -> f64;
        
        /// 计算胜利调整系数
        fn victory_adjustment(&self, shift: f64, opponents_count: usize) -> f64;
    }

    /// 默认的ELO配置实现
    #[derive(Debug, Clone)]
    pub struct DefaultEloConfig {
        perf: f64,
        k_factor: f64,
    }

    impl DefaultEloConfig {
        /// 创建新的默认ELO配置
        pub fn new(perf: f64, k_factor: f64) -> Self {
            Self { perf, k_factor }
        }
        
        /// 使用推荐值创建默认的ELO配置
        pub fn default() -> Self {
            Self::new(400.0, 70.0)
        }
    }

    impl EloConfig for DefaultEloConfig {
        fn performance_constant(&self) -> f64 {
            self.perf
        }
        
        fn k_factor(&self) -> f64 {
            self.k_factor
        }
        
        fn victory_adjustment(&self, shift: f64, opponents_count: usize) -> f64 {
            shift * opponents_count as f64
        }
    }

    /// ELO计算器
    #[derive(Debug, Clone)]
    pub struct EloCalculator<C: EloConfig> {
        config: C,
    }

    impl<C: EloConfig> EloCalculator<C> {
        /// 创建新的ELO计算器
        pub fn new(config: C) -> Self {
            Self { config }
        }
        
        /// 计算预期赢率
        pub fn expected_score(&self, rating: i32, opponent_rating: i32) -> f64 {
            1.0 / (1.0 + (10.0f64.powf((opponent_rating - rating) as f64 / self.config.performance_constant())))
        }
        
        /// 计算一个玩家对多个对手的预期赢率
        pub fn expected_score_vs_many(&self, rating: i32, opponents: &[i32]) -> f64 {
            if opponents.is_empty() {
                return 0.5;
            }
            
            let avg_rating = opponents.iter().sum::<i32>() as f64 / opponents.len() as f64;
            self.expected_score(rating, avg_rating.round() as i32)
        }
        
        /// 计算新的评分
        pub fn calculate_new_rating(&self, rating: i32, opponents: &[i32], outcome: MatchOutcome) -> i32 {
            if opponents.is_empty() {
                return rating;
            }
            
            let expected = self.expected_score_vs_many(rating, opponents);
            
            // 实际得分
            let actual_score = match outcome {
                MatchOutcome::Victory => 1.0,
                MatchOutcome::Defeat => 0.0,
                // 可以在此扩展其他结果
            };
            
            let score_diff = actual_score - expected;
            let mut shift = self.config.k_factor() * score_diff;
            
            // 胜利时的额外调整
            if outcome == MatchOutcome::Victory {
                shift = self.config.victory_adjustment(shift, opponents.len());
            }
            
            (rating as f64 + shift).round() as i32
        }
        
        /// 如果获胜，计算新的评分
        pub fn if_won(&self, rating: i32, opponents: &[i32]) -> i32 {
            self.calculate_new_rating(rating, opponents, MatchOutcome::Victory)
        }
        
        /// 如果失败，计算新的评分
        pub fn if_lost(&self, rating: i32, opponents: &[i32]) -> i32 {
            self.calculate_new_rating(rating, opponents, MatchOutcome::Defeat)
        }
    }
    
    /// 便捷函数，创建默认的ELO计算器
    pub fn create_default_calculator() -> EloCalculator<DefaultEloConfig> {
        EloCalculator::new(DefaultEloConfig::default())
    }
    
    /// 使用默认计算器计算胜利后的新评分
    pub fn if_won(rating: i32, opponents: &[i32]) -> i32 {
        create_default_calculator().if_won(rating, opponents)
    }
    
    /// 使用默认计算器计算失败后的新评分
    pub fn if_lost(rating: i32, opponents: &[i32]) -> i32 {
        create_default_calculator().if_lost(rating, opponents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shuffle() {
        let original = vec![1, 2, 3, 4, 5];
        let shuffled = shuffle(&original);
        
        // 检查长度相同
        assert_eq!(original.len(), shuffled.len());
        
        // 检查所有元素都在
        for item in &original {
            assert!(shuffled.contains(item));
        }
        
        // 注意：理论上有极小概率洗牌后顺序完全不变，但这个测试主要是验证基本功能
    }

    #[test]
    fn test_split_into_chunks() {
        let array = vec![1, 2, 3, 4, 5, 6, 7];
        
        // 测试正常分块
        let chunks = split_into_chunks(&array, 3);
        assert_eq!(chunks, vec![vec![1, 2, 3], vec![4, 5, 6], vec![7]]);
        
        // 测试块大小大于数组
        let big_chunks = split_into_chunks(&array, 10);
        assert_eq!(big_chunks, vec![vec![1, 2, 3, 4, 5, 6, 7]]);
        
        // 测试空数组
        let empty = Vec::<i32>::new();
        let empty_chunks = split_into_chunks(&empty, 3);
        assert_eq!(empty_chunks, Vec::<Vec<i32>>::new());
    }
    
    #[test]
    fn test_elo_system() {
        use super::elo::{EloCalculator, DefaultEloConfig, MatchOutcome};
        
        // 创建默认的计算器
        let calculator = EloCalculator::new(DefaultEloConfig::default());
        
        // 测试预期胜率计算
        let player_rating = 1500;
        let opponent_rating = 1400;
        let expected = calculator.expected_score(player_rating, opponent_rating);
        assert!(expected > 0.5); // 玩家评分更高，期望胜率应大于0.5
        
        // 测试胜利后的评分变化
        let new_rating = calculator.if_won(player_rating, &[opponent_rating]);
        assert!(new_rating > player_rating); // 胜利应增加评分
        
        // 测试失败后的评分变化
        let new_rating = calculator.if_lost(player_rating, &[opponent_rating]);
        assert!(new_rating < player_rating); // 失败应减少评分
        
        // 测试计算多个对手
        let opponents = vec![1400, 1450, 1500];
        let new_rating = calculator.calculate_new_rating(player_rating, &opponents, MatchOutcome::Victory);
        assert!(new_rating > player_rating); // 胜利应增加评分
    }
    
    #[test]
    fn test_elo_convenience_functions() {
        use super::elo;
        
        let player_rating = 1500;
        let opponents = vec![1400, 1450];
        
        // 测试便捷函数
        let won_rating = elo::if_won(player_rating, &opponents);
        let lost_rating = elo::if_lost(player_rating, &opponents);
        
        assert!(won_rating > player_rating);
        assert!(lost_rating < player_rating);
    }
}
