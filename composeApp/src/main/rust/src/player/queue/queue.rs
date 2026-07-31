use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};

use crate::player::file_scanner::TrackData;

/// Как поступать, дойдя до конца.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum RepeatMode {
    /// Список по кругу: с последней дорожки идём на первую. Так же ведёт себя [`All`] —
    /// список замыкается всегда, и различать эти два случая стало нечем.
    ///
    /// [`All`]: RepeatMode::All
    Off,
    /// Одна дорожка по кругу. На кнопку «следующая» не влияет: нажатие есть нажатие.
    One,
    /// Весь список по кругу. То же, что [`Off`]; вариант остался потому, что он пересекает
    /// границу FFI и его удаление потребовало бы перегенерации биндингов и правки Kotlin.
    ///
    /// [`Off`]: RepeatMode::Off
    All,
}

/// Список дорожек и место в нём.
///
/// Порядок воспроизведения хранится отдельно от списка: перемешивание меняет только его,
/// а список остаётся тем, каким его видит пользователь, — иначе экран прыгал бы под
/// руками при каждом включении перемешивания.
pub struct Queue {
    tracks: Vec<TrackData>,
    /// Номера дорожек в порядке воспроизведения.
    order: Vec<usize>,
    /// Место в `order`, а не в `tracks`.
    position: Option<usize>,
    repeat: RepeatMode,
    shuffle: bool,
}

impl Queue {
    pub fn new() -> Self {
        Queue {
            tracks: Vec::new(),
            order: Vec::new(),
            position: None,
            repeat: RepeatMode::Off,
            shuffle: false,
        }
    }

    /// Задаёт список. `start` — номер дорожки в списке, с которой начинать; `None`
    /// означает «с первой по порядку воспроизведения».
    pub fn set_tracks(&mut self, tracks: Vec<TrackData>, start: Option<usize>) -> Result<()> {
        if let Some(index) = start {
            if index >= tracks.len() {
                return Err(anyhow!(
                    "начать просят с дорожки {}, а их всего {}",
                    index,
                    tracks.len()
                ));
            }
        }

        self.order = (0..tracks.len()).collect();
        self.tracks = tracks;
        if self.shuffle {
            self.reorder(seed_from_clock());
        }

        self.position = match start {
            Some(index) => self.place_of(index),
            None if self.order.is_empty() => None,
            None => Some(0),
        };
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    /// Номер текущей дорожки в списке — тот, который показывают пользователю.
    pub fn current_index(&self) -> Option<usize> {
        self.position.map(|place| self.order[place])
    }

    pub fn current(&self) -> Option<&TrackData> {
        self.current_index().map(|index| &self.tracks[index])
    }

    /// Следующая по нажатию кнопки. Повтор одной дорожки здесь не действует: человек
    /// попросил другую дорожку, а не ту же самую.
    pub fn next(&mut self) -> Option<&TrackData> {
        let place = self.place_after(self.position?)?;
        self.position = Some(place);
        self.current()
    }

    pub fn previous(&mut self) -> Option<&TrackData> {
        let place = self.place_before(self.position?)?;
        self.position = Some(place);
        self.current()
    }

    /// Следующая после того, как дорожка доиграла сама. Здесь повтор одной действует.
    pub fn advance(&mut self) -> Option<&TrackData> {
        if self.repeat == RepeatMode::One && self.position.is_some() {
            return self.current();
        }
        self.next()
    }

    /// Есть ли куда идти дальше. Спрашивают об этом и уведомление на Android, и MPRIS на
    /// десктопе: сами они из списка этого не выводят.
    pub fn has_next(&self) -> bool {
        self.position
            .and_then(|place| self.place_after(place))
            .is_some()
    }

    pub fn has_previous(&self) -> bool {
        self.position
            .and_then(|place| self.place_before(place))
            .is_some()
    }

    /// Переходит к дорожке с таким номером в списке. Это то, что делает нажатие на
    /// песню в списке на экране: порядок воспроизведения при этом не меняется, меняется
    /// только место в нём.
    pub fn jump_to(&mut self, index: usize) -> Result<()> {
        let place = self
            .place_of(index)
            .ok_or_else(|| anyhow!("дорожки {} в списке нет", index))?;
        self.position = Some(place);
        Ok(())
    }

    pub fn repeat(&self) -> RepeatMode {
        self.repeat
    }

    pub fn set_repeat(&mut self, repeat: RepeatMode) {
        self.repeat = repeat;
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle
    }

    /// Включает или выключает перемешивание. Текущая дорожка остаётся текущей: меняется
    /// то, что будет после неё.
    pub fn set_shuffle(&mut self, shuffle: bool) {
        self.set_shuffle_seeded(shuffle, seed_from_clock());
    }

    fn set_shuffle_seeded(&mut self, shuffle: bool, seed: u64) {
        if self.shuffle == shuffle {
            return;
        }
        self.shuffle = shuffle;

        let current = self.current_index();
        if shuffle {
            self.reorder(seed);
        } else {
            self.order = (0..self.tracks.len()).collect();
        }
        self.position = current.and_then(|index| self.place_of(index));
    }

    /// Куда идти после `place`, с учётом повтора всего списка.
    /// Список замкнут: с последней дорожки шаг ведёт на первую, и это не зависит от повтора.
    /// Поэтому шаг есть всегда, пока список не пуст, — и `has_next` с `has_previous` на
    /// непустом списке всегда правда.
    fn place_after(&self, place: usize) -> Option<usize> {
        if place + 1 < self.order.len() {
            Some(place + 1)
        } else if !self.order.is_empty() {
            Some(0)
        } else {
            None
        }
    }

    fn place_before(&self, place: usize) -> Option<usize> {
        if place > 0 {
            Some(place - 1)
        } else if !self.order.is_empty() {
            Some(self.order.len() - 1)
        } else {
            None
        }
    }

    /// Какое место в порядке воспроизведения занимает дорожка с таким номером в списке.
    fn place_of(&self, index: usize) -> Option<usize> {
        self.order.iter().position(|&track| track == index)
    }

    /// Перекладывает порядок воспроизведения перетасовкой Фишера — Йетса.
    fn reorder(&mut self, seed: u64) {
        let mut shuffler = Shuffler::new(seed);
        for place in (1..self.order.len()).rev() {
            self.order.swap(place, shuffler.below(place + 1));
        }
    }
}

/// Генератор псевдослучайных чисел xorshift64*. Своего хватает: очередь перемешивают, а
/// не шифруют. Он же позволяет задать зерно и сделать перемешивание проверяемым.
struct Shuffler {
    state: u64,
}

impl Shuffler {
    fn new(seed: u64) -> Self {
        // Нулевое состояние для xorshift вырожденное: из нуля получается только ноль.
        Shuffler {
            state: if seed == 0 { GOLDEN } else { seed },
        }
    }

    fn next(&mut self) -> u64 {
        let mut state = self.state;
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        self.state = state;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Число меньше `bound`. Остаток от деления даёт едва заметный перекос в сторону
    /// малых чисел; на списке дорожек он неразличим.
    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

/// Зерно из часов: два запуска подряд перемешают по-разному.
const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

fn seed_from_clock() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(since) => since.as_nanos() as u64,
        // Часы раньше 1970 года бывают только на сломанной машине. Последствие —
        // одинаковое перемешивание от запуска к запуску, и больше ничего.
        Err(_) => GOLDEN,
    }
}

#[cfg(test)]
mod tests {
    use super::{Queue, RepeatMode, TrackData};

    fn track(index: usize) -> TrackData {
        TrackData {
            track_url: format!("{index}.mp3"),
            track_name: format!("дорожка {index}"),
            track_index: index as i32,
            is_local: true,
        }
    }

    fn queue_of(count: usize) -> Queue {
        let tracks = (0..count).map(track).collect();
        let mut queue = Queue::new();
        queue.set_tracks(tracks, None).expect("список принят");
        queue
    }

    /// Номера дорожек в том порядке, в каком они будут играть, за столько шагов. Список
    /// замкнут, конца у него нет, поэтому число шагов задаётся здесь.
    fn walk(queue: &mut Queue, steps: usize) -> Vec<usize> {
        let mut seen = Vec::new();
        for _ in 0..steps {
            assert!(queue.next().is_some(), "на непустом списке шаг есть всегда");
            seen.push(queue.current_index().expect("дорожка есть"));
        }
        seen
    }

    #[test]
    fn walks_the_list_and_closes_the_ring() {
        let mut queue = queue_of(3);
        assert_eq!(queue.current_index(), Some(0));
        assert!(queue.has_next());
        assert!(queue.has_previous(), "назад с первой ведёт к последней");

        // Три шага по списку из трёх: круг замкнулся, снова первая.
        assert_eq!(walk(&mut queue, 3), vec![1, 2, 0]);
        assert!(queue.has_next());
        queue.previous();
        assert_eq!(queue.current_index(), Some(2), "назад с первой — последняя");
    }

    #[test]
    fn end_of_the_list_goes_to_the_first_track() {
        let mut queue = queue_of(3);
        queue.jump_to(2).expect("дорожка есть");
        // Последняя дорожка доиграла сама — дальше первая, а не тишина.
        assert!(queue.advance().is_some(), "с конца списка идём на первую");
        assert_eq!(queue.current_index(), Some(0));
    }

    #[test]
    fn empty_queue_answers_nothing() {
        let mut queue = Queue::new();
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.current_index(), None);
        assert!(queue.next().is_none());
        assert!(queue.previous().is_none());
        assert!(queue.advance().is_none());
        assert!(!queue.has_next());
        assert!(!queue.has_previous());
    }

    #[test]
    fn repeat_all_is_the_same_as_off() {
        let mut queue = queue_of(3);
        queue.set_repeat(RepeatMode::All);
        assert_eq!(walk(&mut queue, 3), vec![1, 2, 0]);
        assert!(queue.has_previous(), "назад с первой ведёт к последней");
    }

    #[test]
    fn repeat_one_holds_the_track_but_not_the_button() {
        let mut queue = queue_of(3);
        queue.set_repeat(RepeatMode::One);
        // Дорожка доиграла сама — та же ещё раз.
        assert!(queue.advance().is_some(), "дорожка должна повториться");
        assert_eq!(queue.current_index(), Some(0));
        // Нажали «следующая» — всё равно следующая.
        queue.next();
        assert_eq!(queue.current_index(), Some(1));
    }

    #[test]
    fn start_index_decides_where_to_begin() {
        let mut queue = Queue::new();
        let tracks = (0..4).map(track).collect();
        queue.set_tracks(tracks, Some(2)).expect("список принят");
        assert_eq!(queue.current_index(), Some(2));
        assert_eq!(walk(&mut queue, 2), vec![3, 0], "за последней идёт первая");
    }

    #[test]
    fn start_beyond_the_list_is_an_error() {
        let mut queue = Queue::new();
        assert!(queue.set_tracks(vec![track(0)], Some(1)).is_err());
    }

    #[test]
    fn jump_goes_to_the_asked_track() {
        let mut queue = queue_of(4);
        queue.jump_to(2).expect("дорожка есть");
        assert_eq!(queue.current_index(), Some(2));
        assert!(queue.jump_to(4).is_err(), "дорожки 4 в списке из четырёх нет");
        assert_eq!(queue.current_index(), Some(2), "неудачный переход ничего не меняет");
    }

    #[test]
    fn shuffle_keeps_the_track_and_the_set() {
        let mut queue = queue_of(100);
        queue.next();
        queue.next();
        let playing = queue.current_index().expect("дорожка есть");

        queue.set_shuffle_seeded(true, 12_345);
        assert_eq!(queue.current_index(), Some(playing), "играть должна та же дорожка");
        assert_eq!(queue.len(), 100, "список не должен меняться");

        // Порядок стал другим, но дорожки в нём те же и каждая по одному разу.
        let mut order = queue.order.clone();
        assert_ne!(order, (0..100).collect::<Vec<_>>());
        order.sort_unstable();
        assert_eq!(order, (0..100).collect::<Vec<_>>());

        // Выключили — порядок вернулся, дорожка осталась.
        queue.set_shuffle_seeded(false, 0);
        assert_eq!(queue.order, (0..100).collect::<Vec<_>>());
        assert_eq!(queue.current_index(), Some(playing));
    }
}
