function ensureReadableStreamAsyncIterator() {
  if (
    typeof ReadableStream === 'undefined' ||
    Symbol.asyncIterator in ReadableStream.prototype
  ) {
    return;
  }

  Object.defineProperty(ReadableStream.prototype, Symbol.asyncIterator, {
    configurable: true,
    value: function (this: ReadableStream<unknown>): AsyncIterableIterator<unknown> {
      const reader = this.getReader();
      let released = false;

      const releaseLock = () => {
        if (!released) {
          released = true;
          reader.releaseLock();
        }
      };

      const iterator: AsyncIterableIterator<unknown> = {
        async next() {
          const result = await reader.read();
          if (result.done) releaseLock();
          return result;
        },
        async return() {
          try {
            await reader.cancel();
          } finally {
            releaseLock();
          }
          return { done: true, value: undefined };
        },
        [Symbol.asyncIterator]() {
          return iterator;
        }
      };

      return iterator;
    }
  });
}

ensureReadableStreamAsyncIterator();

// Client-side error handling
export function handleError({ error, event }: { error: any; event: any }) {
  console.error('Client error:', error);
  return {
    message: 'An error occurred',
    code: error?.code ?? 'UNKNOWN'
  };
}
