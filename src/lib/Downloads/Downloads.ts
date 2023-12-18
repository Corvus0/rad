export interface DownloadInput {
  url: string;
  op: string;
  sub: string;
}

export enum DownloadStatus {
  Initial = "Initial",
  Downloading = "Downloading",
  Completed = "Completed",
  Failed = "Failed",
}

export interface DownloadOutput {
  id: number;
  input: DownloadInput;
  audio: string;
  title: string;
  status: DownloadStatus;
  failure: string | null;
}
