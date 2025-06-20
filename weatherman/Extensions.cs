namespace weatherman;

public static class EnumerableExtensions
{
    public static IEnumerable<double> FillNullsWithPrevious(this IEnumerable<double?> source)
    {
        double lastValue = 0.0;
        bool hasSeenValue = false;

        foreach (var item in source)
        {
            if (item.HasValue)
            {
                lastValue = item.Value;
                hasSeenValue = true;
                yield return lastValue;
            }
            else if (hasSeenValue)
            {
                // Only yield if we have seen a previous value
                yield return lastValue;
            }
            // If null and no previous value, skip (ignore)
        }
    }
}