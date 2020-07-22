use x86_64::{structures::paging::PageTable, PhysAddr, VirtAddr};
use x86_64::structures::paging::OffsetPageTable;

pub trait IntoPhysAddr {
    fn into_physical_addr(self, physical_offset: VirtAddr) -> Option<PhysAddr>;
}

impl IntoPhysAddr for VirtAddr {
    fn into_physical_addr(self, physical_offset: VirtAddr) -> Option<PhysAddr> {
        let (frame, _) = x86_64::registers::control::Cr3::read();
        let frame = [
            self.p4_index(),
            self.p3_index(),
            self.p2_index(),
            self.p1_index(),
        ]
        .iter()
        .fold(Some(frame), |frame, index| match frame {
            None => None,
            Some(frame) => {
                let virt = physical_offset + frame.start_address().as_u64();
                let table = unsafe { &*virt.as_ptr::<PageTable>() };

                let entry = &table[*index];
                match entry.frame() {
                    Ok(frame) => Some(frame),
                    Err(_) => None,
                }
            }
        });

        frame.map(|frame| frame.start_address() + u64::from(self.page_offset()))
    }
}

/// Level 4 table.
fn level_4_table(physical_offset: VirtAddr) -> &'static mut PageTable {
    let (frame, _) = x86_64::registers::control::Cr3::read();

    let start = frame.start_address();
    let virt_start = physical_offset + start.as_u64();

    // this is the place where the page table exists
    unsafe { &mut *virt_start.as_mut_ptr::<PageTable>() }
}

pub fn init(physical_offset: VirtAddr) -> OffsetPageTable<'static> {
    let table = level_4_table(physical_offset);
    unsafe { OffsetPageTable::new(table, physical_offset) }
}
