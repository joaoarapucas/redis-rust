-- date formatter

local days_in_month = { 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 }

local function is_leap_year(year)
    if year % 400 == 0 then
        return true
    end
    if year % 100 == 0 then
        return false
    end
    return year % 4 == 0
end

local function is_valid_format(text)
    if #text ~= 10 then
        return false
    end
    if text:sub(5, 5) ~= "-" or text:sub(8, 8) ~= "-" then
        return false
    end
    for i = 1, 10 do
        if i ~= 5 and i ~= 8 then
            local character = text:sub(i, i)
            if character < "0" or character > "9" then
                return false
            end
        end
    end
    return true
end

local function split_date(text)
    local year = tonumber(text:sub(1, 4))
    local month = tonumber(text:sub(6, 7))
    local day = tonumber(text:sub(9, 10))
    return year, month, day
end

local function date_exists(year, month, day)
    if month < 1 or month > 12 then
        return false
    end

    local day_limit = days_in_month[month]
    if month == 2 and is_leap_year(year) then
        day_limit = 29
    end

    if day < 1 or day > day_limit then
        return false
    end

    return true
end

return {
    prefix = "data_",

    add = function(key, value)
        if not is_valid_format(value) then
            error("data deve estar no formato aaaa-mm-dd", 0)
        end

        local year, month, day = split_date(value)
        if not date_exists(year, month, day) then
            error("data inexistente no calendario", 0)
        end

        return nil -- keep the original value (aaaa-mm-dd)
    end,

    get = function(key, value)
        local year = value:sub(1, 4)
        local month = value:sub(6, 7)
        local day = value:sub(9, 10)
        return day .. "/" .. month .. "/" .. year
    end,
}
