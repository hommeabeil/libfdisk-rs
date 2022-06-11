use serde::Deserialize;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

use libfdisk::context::Context;
use libfdisk::label::DiskLabel;

const DEFAULT_TYPE: &str = "0FC63DAF-8483-4772-8E79-3D69D8477DE4";
const DEFAULT_GPT_LBA: u64 = 34;

#[derive(Debug, Deserialize)]
struct SFdiskDump {
    #[serde(rename = "partitiontable")]
    partition_table: SFdiskPartitionTable,
}

#[derive(Debug, Deserialize)]
struct SFdiskPartitionTable {
    label: String,
    id: String,
    device: String,
    unit: String,
    #[serde(rename = "firstlba")]
    first_lba: u64,
    #[serde(rename = "lastlba")]
    last_lba: u64,
    grain: String,
    #[serde(rename = "sectorsize")]
    sector_size: u32,
    partitions: Option<Vec<SFdiskPartition>>,
}

#[derive(Debug, Deserialize)]
struct SFdiskPartition {
    node: String,
    start: u64,
    size: u64,
    #[serde(rename = "type")]
    _type: String,
    uuid: String,
}

fn sfdisk_dump(path: &std::path::Path) -> std::io::Result<SFdiskDump> {
    let result = Command::new("/usr/sbin/sfdisk")
        .args(["--json", "--dump", path.to_str().unwrap()])
        .output()?;
    let stdout = std::str::from_utf8(&result.stdout).unwrap();

    let out: SFdiskDump = serde_json::from_str(&stdout)?;
    Ok(out)
}

fn create_file() -> std::path::PathBuf {
    let file = NamedTempFile::new().unwrap();
    let (mut file2, path) = file.keep().unwrap();
    // let mut file3 = file2.open().unwrap();
    let zeros = vec![0; 65528];
    file2.write_all(&zeros).unwrap();
    path
}

#[test]
fn create_gpt_label() {
    let path = create_file();

    let context = Context::new();
    context.assign_device(&path, false).unwrap();
    context.create_disklabel(DiskLabel::Gpt).unwrap();
    context.write_disklabel().unwrap();
    context.deassign_device(false).unwrap();

    let label = context.get_label("").unwrap();
    assert_eq!("gpt", label.get_name().unwrap());

    let result = sfdisk_dump(&path).unwrap();

    assert_eq!("gpt", result.partition_table.label)
}

#[test]
fn create_a_single_partition() {
    let path = create_file();

    let context = Context::new();
    context.assign_device(&path, false).unwrap();
    context.create_disklabel(DiskLabel::Gpt).unwrap();

    let partition = libfdisk::Partition::new();
    partition.set_start(context.first_lba()).unwrap();
    partition
        .set_size(context.last_lba() - context.first_lba() + 1)
        .unwrap();
    partition.set_partno(0).unwrap();
    context.set_partition(0, &partition).unwrap();

    context.write_disklabel().unwrap();
    context.deassign_device(false).unwrap();

    let dump = sfdisk_dump(&path).unwrap();

    let parts = dump.partition_table.partitions.unwrap();
    let first_part = parts.get(0).unwrap();

    assert_eq!(DEFAULT_GPT_LBA, first_part.start);
    assert_eq!(60, first_part.size);
    assert_eq!(DEFAULT_TYPE, first_part._type)
}

#[test]
fn delete_all_partitions() {
    let path = create_file();

    let context = Context::new();
    context.assign_device(&path, false).unwrap();
    context.create_disklabel(DiskLabel::Gpt).unwrap();

    let partition = libfdisk::Partition::new();
    partition.set_start(context.first_lba()).unwrap();
    partition.set_size(10).unwrap();
    partition.set_partno(0).unwrap();
    context.set_partition(0, &partition).unwrap();

    context.delete_all_partitions().unwrap();

    context.write_disklabel().unwrap();
    context.deassign_device(false).unwrap();

    let dump = sfdisk_dump(&path).unwrap();

    assert!(dump.partition_table.partitions.is_none())
}

#[test]
fn delete_a_partition() {
    let path = create_file();

    let context = Context::new();
    context.assign_device(&path, false).unwrap();
    context.create_disklabel(DiskLabel::Gpt).unwrap();

    let partition = libfdisk::Partition::new();
    partition.set_start(context.first_lba()).unwrap();
    partition.set_size(10).unwrap();
    partition.set_partno(0).unwrap();
    context.set_partition(0, &partition).unwrap();

    context.delete_partition(0).unwrap();

    context.write_disklabel().unwrap();
    context.deassign_device(false).unwrap();

    let dump = sfdisk_dump(&path).unwrap();

    assert!(dump.partition_table.partitions.is_none())
}

#[test]
fn cannot_set_size_bigger_then_lba() {
    let path = create_file();

    let context = Context::new();
    context.assign_device(&path, false).unwrap();
    context.create_disklabel(DiskLabel::Gpt).unwrap();

    let partition = libfdisk::Partition::new();
    partition.set_start(context.first_lba()).unwrap();
    partition.set_size(1000).unwrap();
    assert!(context.set_partition(0, &partition).is_err())
}

#[test]
fn use_table() {
    let path = create_file();

    let context = Context::new();
    context.assign_device(&path, false).unwrap();
    context.create_disklabel(DiskLabel::Gpt).unwrap();

    let table = context.get_partitions().unwrap();
    let mut partition = libfdisk::Partition::new();
    partition.set_start(context.first_lba()).unwrap();
    partition.set_size(10).unwrap();
    partition.set_partno(0).unwrap();
    table.add_partition(&mut partition).unwrap();

    assert_eq!(1, table.nents());

    context.apply_table(&table).unwrap();
    context.write_disklabel().unwrap();
    context.deassign_device(false).unwrap();

    let dump = sfdisk_dump(&path).unwrap();

    assert!(!dump.partition_table.partitions.is_none())
}
