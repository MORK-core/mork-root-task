use alloc::vec;
use alloc::vec::Vec;
use core::ptr;
use elf::abi::{PF_R, PF_W, PF_X, PT_LOAD};
use elf::ElfBytes;
use elf::endian::AnyEndian;
use elf::segment::ProgramHeader;
use lazy_init::LazyInit;
use mork_common::constants::{CNodeSlot, ObjectType, PAGE_SIZE_NORMAL};
use mork_common::hal::{UserContext, UserContextTrait};
use mork_common::mork_user_log;
use mork_common::syscall::message_info::ResponseLabel;
use mork_common::types::{ResultWithErr, VMRights};
use mork_common::utils::alignas::{align_down, align_up};
use mork_user_lib::mork_task::{mork_alloc_object, mork_delete_object, mork_thread_resume, mork_thread_set_space, mork_thread_suspend, mork_thread_write_registers};
use mork_user_lib::mork_mm::{mork_map_frame_anyway, mork_unmap_frame};
use crate::auto_gen::TEST_META_INFOS;

#[allow(dead_code)]
unsafe extern "C" {
    fn tests_data_start();
    fn tests_data_end();
}

#[allow(dead_code)]
struct CaseInfo<'a> {
    case_name: &'static str,
    case_elf: ElfBytes<'a, AnyEndian>,
}

#[allow(dead_code)]
struct TestInfo<'a> {
    cases: Vec<CaseInfo<'a>>,
}

static TEST_INFO: LazyInit<TestInfo> = LazyInit::new();

pub fn parse() -> ResultWithErr<ResponseLabel> {
    let mut tests = Vec::new();
    let mut offset = tests_data_start as usize;
    for test_meta_info in TEST_META_INFOS {
        let elf_data = unsafe {
            core::slice::from_raw_parts(offset as *const u8, test_meta_info.file_size)
        };
        let elf = ElfBytes::<AnyEndian>::minimal_parse(elf_data).unwrap();

        tests.push(CaseInfo {
            case_name: test_meta_info.test_name,
            case_elf: elf,
        });
        offset += test_meta_info.file_size;
    }

    TEST_INFO.init_by(TestInfo {
        cases: tests,
    });
    Ok(())
}

pub fn run() -> ResultWithErr<ResponseLabel> {
    // map frames to root task vspace
    for case in &TEST_INFO.cases {
        mork_user_log!(info, "start run test case: {}...", case.case_name);
        run_signal_test(&case.case_elf)?
    }
    Ok(())
}

fn run_signal_test(test: &ElfBytes<AnyEndian>) -> ResultWithErr<ResponseLabel> {
    let segments = test.segments().unwrap();
    let mut alloc_object = Vec::new();
    let task = mork_alloc_object(CNodeSlot::CapInitThread as usize, ObjectType::Thread)?;
    let vspace = mork_alloc_object(CNodeSlot::CapInitThread as usize, ObjectType::PageTable)?;
    for segment in segments {
        let mut alloc_frame = alloc_frame_and_copy_data(segment, test)?;
        let mut alloc_page_table = init_vspace_for_case(vspace, segment, &alloc_frame)?;
        alloc_object.append(&mut alloc_frame);
        alloc_object.append(&mut alloc_page_table);
    }
    let start_entry = test.ehdr.e_entry as usize;
    let mut user_context = UserContext::new();
    user_context.set_next_ip(start_entry);
    mork_thread_write_registers(task, &user_context)?;

    mork_thread_set_space(task, vspace)?;
    mork_thread_resume(task)?;
    mork_thread_suspend(CNodeSlot::CapInitThread as usize).unwrap();


    for object in alloc_object {
        mork_delete_object(CNodeSlot::CapInitThread as usize, object)?;
    }
    mork_delete_object(CNodeSlot::CapInitThread as usize, task)?;
    mork_delete_object(CNodeSlot::CapInitThread as usize, vspace)?;
    Ok(())
}

fn init_vspace_for_case(vspace: usize, segment: ProgramHeader, frames: &Vec<usize>)
    -> Result<Vec<usize>, ResponseLabel> {
    if segment.p_type != PT_LOAD {
        return Ok(vec![]);
    }
    let alloc_page_table = Vec::new();
    let mut vaddr = segment.p_vaddr as usize;

    let mut rights = VMRights::empty();
    if segment.p_flags & PF_X != 0 {
        rights |= VMRights::X;
    }
    if segment.p_flags & PF_W != 0 {
        rights |= VMRights::W;
    }
    if segment.p_flags & PF_R != 0 {
        rights |= VMRights::R;
    }
    let mut wrapper = Some(alloc_page_table);
    for frame in frames {
        mork_map_frame_anyway(
            CNodeSlot::CapInitThread as usize, vspace, *frame, vaddr, rights,
            &mut wrapper,
        )?;
        vaddr += PAGE_SIZE_NORMAL;
    }
    Ok(wrapper.unwrap())
}

fn alloc_frame_and_copy_data(segment: ProgramHeader, elf: &ElfBytes<AnyEndian>)
    -> Result<Vec<usize>, ResponseLabel> {
    let elf_data_vaddr_on_root_task: usize = 0x1000_0000;
    let mut map_start = elf_data_vaddr_on_root_task;
    let mut frames = Vec::new();
    if segment.p_type != PT_LOAD {
        return Ok(frames);
    }
    mork_user_log!(info,
        "Segment: offset=0x{:#x}, vaddr={:#x}, file size={:#x}, mem size: {:#x}, flags={:x}",
        segment.p_offset, segment.p_vaddr, segment.p_filesz, segment.p_memsz, segment.p_flags
    );
    let need_copy = segment.p_filesz == segment.p_memsz;
    let mut vaddr = align_down(segment.p_vaddr as usize, PAGE_SIZE_NORMAL);
    let end = align_up(segment.p_vaddr as usize + segment.p_memsz as usize, PAGE_SIZE_NORMAL);
    while vaddr < end {
        let frame = mork_alloc_object(CNodeSlot::CapInitThread as usize, ObjectType::Frame4K)?;
        frames.push(frame);
        if need_copy {
            mork_map_frame_anyway(
                CNodeSlot::CapInitThread as usize,
                CNodeSlot::CapInitVSpace as usize,
                frame,
                map_start,
                VMRights::R | VMRights::W,
                &mut None,
            )?;
            map_start += PAGE_SIZE_NORMAL;
        }
        vaddr += PAGE_SIZE_NORMAL;
    }
    if need_copy {
        let segment_data = elf.segment_data(&segment).map_err(|_| ResponseLabel::InvalidParam )?;
        if segment_data.len() > frames.len() * PAGE_SIZE_NORMAL {
            mork_user_log!(error, "Invalid data size");
            return Err(ResponseLabel::InvalidParam);
        }
        let dest_ptr = elf_data_vaddr_on_root_task as *mut u8;
        if dest_ptr.is_null() {
            mork_user_log!(error, "Invalid dest ptr");
            return Err(ResponseLabel::InvalidParam);
        }
        unsafe {
            ptr::copy_nonoverlapping(segment_data.as_ptr(), dest_ptr, segment_data.len());
        }
    }

    if need_copy {
        for frame in &frames {
            mork_unmap_frame(CNodeSlot::CapInitVSpace as usize, *frame)?;
        }
    }
    Ok(frames)
}