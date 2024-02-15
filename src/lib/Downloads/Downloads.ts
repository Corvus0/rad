export interface DownloadInput {
  url: string;
  op: string;
  sub: string;
}

export interface DownloadInfo {
  audio: string;
  title: string;
  headers: Map<string, string>;
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
  info: DownloadInfo;
  status: DownloadStatus;
  failure: string | null;
}
